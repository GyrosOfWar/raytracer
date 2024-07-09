pub type Point3<T> = nalgebra::Point3<T>;
pub type Vec3<T> = nalgebra::Vector3<T>;

pub fn reflect(v: Vec3<f32>, n: Vec3<f32>) -> Vec3<f32> {
    v - n * Vec3::dot(&v, &n) * 2.0
}

pub fn refract(uv: Vec3<f32>, n: Vec3<f32>, etai_over_etat: f32) -> Vec3<f32> {
    let cos_theta = (-uv).dot(&n).min(1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parallel = n * -(1.0 - r_out_perp.norm_squared()).abs().sqrt();
    r_out_perp + r_out_parallel
}

pub mod random {
    use crate::random::{random, random_range};

    use super::Vec3;

    pub fn gen() -> Vec3<f32> {
        Vec3::new(random(), random(), random())
    }

    pub fn gen_range(min: f32, max: f32) -> Vec3<f32> {
        Vec3::new(
            random_range(min, max),
            random_range(min, max),
            random_range(min, max),
        )
    }

    pub fn gen_unit_sphere() -> Vec3<f32> {
        loop {
            let p = gen_range(-1.0, 1.0);
            if p.dot(&p) < 1.0 {
                return p;
            }
        }
    }

    pub fn gen_unit_disk() -> Vec3<f32> {
        loop {
            let p = Vec3::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0), 0.0);
            if p.dot(&p) < 1.0 {
                return p;
            }
        }
    }

    pub fn gen_unit_vector() -> Vec3<f32> {
        gen_unit_sphere().normalize()
    }

    #[allow(unused)]
    pub fn gen_on_hemisphere(normal: Vec3<f32>) -> Vec3<f32> {
        let on_unit_sphere = gen_unit_vector();
        if on_unit_sphere.dot(&normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }
}
