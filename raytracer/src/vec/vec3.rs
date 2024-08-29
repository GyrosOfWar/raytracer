use super::generic::{Point3f, Vec3f};

pub type Vec3 = Vec3f;

pub type Point3 = Point3f;

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - n * Vec3::dot(&v, n) * 2.0
}

pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parallel = n * -(1.0 - r_out_perp.length_squared()).abs().sqrt();
    r_out_perp + r_out_parallel
}

#[cfg(test)]
mod tests {
    use crate::vec::Vec3;

    #[test]
    fn test_vec3_dot() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a.dot(b), 4.0 + 10.0 + 18.0);
    }
}
