use num_traits::{Float, One};

use crate::{
    ppm::Color,
    ray::Ray,
    vec3::{Point3, Vec3},
};

pub fn hit_sphere(ray: &Ray<f32>, center: Point3<f32>, radius: f32) -> bool {
    let oc = center - ray.origin;
    let a = ray.direction.dot(ray.direction);
    let b = ray.direction.dot(oc) * -2.0;
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant >= 0.0
}

pub fn ray_color(ray: &Ray<f32>) -> Color {
    if hit_sphere(ray, Point3::new(0.0, 0.0, -1.0), 0.5) {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        }
    } else {
        let direction = ray.direction.unit();
        let t = 0.5 * (direction.y + 1.0);
        Vec3::one().lerp(Vec3::new(0.5, 0.7, 1.0), t).into()
    }
}
