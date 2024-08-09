use core::f32;

use crate::vec::{Mat3, Vec3};

pub fn safe_sqrt(u: f32) -> f32 {
    u.max(0.0).sqrt()
}

pub fn abs_cos_theta(v: Vec3) -> f32 {
    v.z.abs()
}

pub fn lerp(x: f32, a: f32, b: f32) -> f32 {
    (1.0 - x) * a + x * b
}

pub fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

/// Evaluate a polynomial using Horner's rule. Coefficients are interpreted
/// in highest-to-lowest order in terms of power.
pub fn evaluate_polynomial(coefficients: &[f32], x: f32) -> f32 {
    let mut result = coefficients[0];

    for coeff in coefficients.iter().copied().skip(1) {
        result = result.mul_add(x, coeff);
    }

    result
}

pub fn linear_least_squares(a: &[Vec3], b: &[Vec3]) -> Mat3 {
    let mut at_a = Mat3::ZERO;
    let mut at_b = Mat3::ZERO;

    for i in 0..3 {
        for j in 0..3 {
            for r in 0..a.len() {
                at_a.set(i, j, a[r].get(i) * a[r].get(j));
                at_b.set(i, j, a[r].get(i) * b[r].get(j));
            }
        }
    }

    let at_a_i = at_a.inverse();
    (at_a_i * at_b).transpose()
}

#[inline(always)]
pub fn square(x: f32) -> f32 {
    x * x
}

#[cfg(test)]
mod tests {
    use crate::math::evaluate_polynomial;

    #[test]
    fn test_evaluate_polynomial() {
        let coefficients = &[0.0, 0.0, 0.0];
        assert_eq!(evaluate_polynomial(coefficients, 1.0), 0.0);

        let coefficients = &[3.0, 2.0, 1.0];
        assert_eq!(evaluate_polynomial(coefficients, 1.0), 6.0);

        let coefficients = &[2.0, -6.0, 2.0, -1.0];
        assert_eq!(evaluate_polynomial(coefficients, 3.0), 5.0);
    }
}

#[derive(Debug, Clone)]
pub struct DirectionCone {
    w: Vec3,
    cos_theta: f32,
}

impl Default for DirectionCone {
    fn default() -> Self {
        Self {
            w: Default::default(),
            cos_theta: f32::INFINITY,
        }
    }
}

impl DirectionCone {
    pub fn new(w: Vec3, cos_theta: f32) -> Self {
        DirectionCone {
            w: w.normalized(),
            cos_theta,
        }
    }

    pub fn from_direction(w: Vec3) -> Self {
        Self::new(w, 1.0)
    }

    pub fn is_empty(&self) -> bool {
        self.cos_theta.is_infinite()
    }

    pub fn entire_sphere() -> Self {
        DirectionCone {
            w: Vec3::new(0.0, 0.0, 1.0),
            cos_theta: -1.0,
        }
    }

    pub fn is_inside(&self, vec: Vec3) -> bool {
        // for the angle to be smaller, the cosine must be larger.
        !self.is_empty() && self.w.dot(vec.normalized()) >= self.cos_theta
    }
}
