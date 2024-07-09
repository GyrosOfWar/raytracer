use std::sync::Arc;

use super::{get_id, HitRecord, Hittable};
use crate::{
    aabb::Aabb,
    material::Material,
    range::Range,
    ray::Ray,
    texture::TextureCoordinates,
    vec3::{Point3, Vec3},
};

#[derive(Debug)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    pub material: Arc<Material>,
    bounding_box: Aabb,
    id: u32,
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32, material: Arc<Material>) -> Self {
        let radius_vec = Vec3::new(radius, radius, radius);
        let bounding_box = Aabb::from_points(center - radius_vec, center + radius_vec);
        let id = get_id();

        Sphere {
            center,
            radius,
            material,
            bounding_box,
            id,
        }
    }
}

/// p: a given point on the sphere of radius one, centered at the origin.
/// u: returned value [0,1] of angle around the Y axis from X=-1.
/// v: returned value [0,1] of angle from Y=-1 to Y=+1.
///     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
///     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
///     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>
fn get_uv(p: Point3<f32>) -> TextureCoordinates {
    use std::f32::consts::PI;

    let theta = (-p.y).acos();
    let phi = f32::atan2(-p.z, p.x) + PI;

    TextureCoordinates {
        u: phi / (2.0 * PI),
        v: theta / PI,
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord> {
        let oc = self.center - ray.origin;
        let a = ray.direction.norm_squared();
        let h = ray.direction.dot(&oc);
        let c = oc.norm_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            None
        } else {
            let sqrtd = discriminant.sqrt();
            let mut root = (h - sqrtd) / a;
            if !hit_range.surrounds(root) {
                root = (h + sqrtd) / a;
                if !hit_range.surrounds(root) {
                    return None;
                }
            }
            let point = ray.evaluate(root);
            let outward_normal = (point - self.center) / self.radius;
            Some(HitRecord::new(
                ray,
                outward_normal,
                point,
                root,
                self.material.clone(),
                get_uv(point),
            ))
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bounding_box
    }

    fn id(&self) -> u32 {
        self.id
    }
}
