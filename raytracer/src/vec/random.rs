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
