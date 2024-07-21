use glam::Vec3A;

pub fn safe_sqrt(u: f32) -> f32 {
    u.max(0.0).sqrt()
}

pub fn abs_cos_theta(v: Vec3A) -> f32 {
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

    for coeff in coefficients.iter().skip(1) {
        result = result.mul_add(x, *coeff);
    }

    result
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
