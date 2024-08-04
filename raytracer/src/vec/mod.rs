mod macros;
mod mat3;
mod mat4;
mod vec2;
mod vec3;

pub use mat3::*;
pub use mat4::*;
pub use vec2::*;
pub use vec3::*;

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

pub fn point2(x: f32, y: f32) -> Point2 {
    Point2::new(x, y)
}

pub fn point3(x: f32, y: f32, z: f32) -> Point3 {
    Point3::new(x, y, z)
}

pub fn ivec2(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

pub fn uvec2(x: u32, y: u32) -> UVec2 {
    UVec2::new(x, y)
}

#[derive(Debug, Copy, Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - n * Vec3::dot(&v, n) * 2.0
}

pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parallel = n * -(1.0 - r_out_perp.length_squared()).abs().sqrt();
    r_out_perp + r_out_parallel
}

pub mod random {
    use super::Vec3;
    use crate::random::{random, random_range};
    use crate::vec::Vec2;

    pub fn gen() -> Vec3 {
        Vec3::new(random(), random(), random())
    }

    pub fn gen_2d() -> Vec2 {
        Vec2::new(random(), random())
    }

    pub fn gen_range(min: f32, max: f32) -> Vec3 {
        Vec3::new(
            random_range(min, max),
            random_range(min, max),
            random_range(min, max),
        )
    }

    pub fn gen_unit_sphere() -> Vec3 {
        loop {
            let p = gen_range(-1.0, 1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn gen_unit_disk() -> Vec3 {
        loop {
            let p = Vec3::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0), 0.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn gen_unit_vector() -> Vec3 {
        gen_unit_sphere().normalized()
    }

    pub fn gen_on_hemisphere(normal: Vec3) -> Vec3 {
        let on_unit_sphere = gen_unit_vector();
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }
}
