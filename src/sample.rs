use std::f32::consts::{FRAC_1_PI, FRAC_PI_2, FRAC_PI_4};

use glam::{Vec2, Vec3A};

use crate::math;

pub fn sample_uniform_disk_concentric(u: Vec2) -> Vec2 {
    let u_offset = 2.0 * u - Vec2::ONE;
    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Vec2::ONE;
    }

    let (theta, r) = if u_offset.x.abs() > u_offset.y.abs() {
        (u_offset.x, FRAC_PI_4 * (u_offset.x / u_offset.y))
    } else {
        (
            u_offset.y,
            FRAC_PI_2 - FRAC_PI_4 * (u_offset.x / u_offset.y),
        )
    };

    r * Vec2::new(theta.cos(), theta.sin())
}

pub fn cosine_hemisphere(u: Vec2) -> Vec3A {
    let d = sample_uniform_disk_concentric(u);
    let z = math::safe_sqrt(1.0 - d.x.sqrt() - d.y.sqrt());

    Vec3A::new(d.x, d.y, z)
}

pub fn cosine_hemisphere_pdf(cos_theta: f32) -> f32 {
    cos_theta * FRAC_1_PI
}
