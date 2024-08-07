use std::f32::consts::{FRAC_1_PI, FRAC_PI_2, FRAC_PI_4};

use crate::math::{self, square};
use crate::random::random;
use crate::vec::{Vec2, Vec3};

pub fn sample_uniform_disk_concentric(u: Vec2) -> Vec2 {
    let u_offset = 2.0 * u - Vec2::ONE;
    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Vec2::ONE;
    }

    let (theta, r) = if u_offset.x.abs() > u_offset.y.abs() {
        (u_offset.x, FRAC_PI_4 * (u_offset.y / u_offset.x))
    } else {
        (
            u_offset.y,
            FRAC_PI_2 - FRAC_PI_4 * (u_offset.x / u_offset.y),
        )
    };

    r * Vec2::new(theta.cos(), theta.sin())
}

pub fn cosine_hemisphere(u: Vec2) -> Vec3 {
    let d = sample_uniform_disk_concentric(u);
    let z = math::safe_sqrt(1.0 - (d.x * d.x) - (d.y * d.y));

    Vec3::new(d.x, d.y, z)
}

pub fn cosine_hemisphere_pdf(cos_theta: f32) -> f32 {
    cos_theta * FRAC_1_PI
}

pub fn sample_visible_wavelengths(u: f32) -> f32 {
    538.0 - 138.888889 * (0.85691062 - 1.82750197 * u).atanh()
}

pub fn visible_wavelengths_pdf(lambda: f32) -> f32 {
    if !(360.0..=830.0).contains(&lambda) {
        0.0
    } else {
        0.0039398042 / square((0.0072 * (lambda - 538.0)).cosh())
    }
}

#[derive(Clone, Debug)]
pub struct Stratified1D {
    count: usize,
    emitted: usize,
}

pub fn stratified_1d(count: usize) -> Stratified1D {
    Stratified1D { count, emitted: 0 }
}

impl Iterator for Stratified1D {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.emitted == self.count {
            return None;
        }

        let u = (self.emitted as f32 + random()) / self.count as f32;
        self.emitted += 1;

        Some(u)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count - self.emitted;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use crate::sample::stratified_1d;
    use crate::util::is_sorted;

    #[test]
    fn test_stratified_1d() {
        let samples: Vec<f32> = stratified_1d(5).collect();
        assert!(is_sorted(&samples));
        assert_eq!(samples.len(), 5);
    }
}
