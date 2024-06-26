use enum_dispatch::enum_dispatch;
use num_traits::One;

use crate::{
    ppm::Color,
    ray::Ray,
    vec3::{Point3, Vec3},
};

pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl Range {
    pub fn contains(&self, value: f32) -> bool {
        value >= self.min && value <= self.max
    }
}

pub struct HitRecord {
    pub point: Point3<f32>,
    pub normal: Vec3<f32>,
    pub distance: f32,
    pub front_facing: bool,
}

impl HitRecord {
    pub fn new(
        ray: &Ray<f32>,
        outward_normal: Vec3<f32>,
        point: Point3<f32>,
        distance: f32,
    ) -> Self {
        let front_facing = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_facing {
            outward_normal
        } else {
            -outward_normal
        };

        HitRecord {
            point,
            normal,
            front_facing,
            distance,
        }
    }
}

#[enum_dispatch]
pub trait Hittable {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord>;
}

#[derive(Debug)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
}

#[enum_dispatch(Hittable)]
#[derive(Debug)]
pub enum Object {
    Sphere(Sphere),
}

#[derive(Debug)]
pub struct World {
    objects: Vec<Object>,
}

impl World {
    pub fn new(objects: Vec<Object>) -> Self {
        Self { objects }
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        let mut record = None;
        let mut closest_so_far = hit_range.max;

        for object in &self.objects {
            let range = Range {
                min: hit_range.min,
                max: closest_so_far,
            };

            if let Some(hit) = object.hit(ray, range) {
                closest_so_far = hit.distance;
                record = Some(hit);
            }
        }

        record
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            None
        } else {
            let sqrtd = discriminant.sqrt();
            let mut root = (h - sqrtd) / a;
            if !hit_range.contains(root) {
                root = (h + sqrtd) / a;
                if !hit_range.contains(root) {
                    return None;
                }
            }
            let point = ray.evaluate(root);
            let outward_normal = (point - self.center) / self.radius;
            Some(HitRecord::new(ray, outward_normal, point, root))
        }
    }
}

pub fn ray_color(ray: &Ray<f32>, world: &impl Hittable) -> Color {
    let intersection = world.hit(
        ray,
        Range {
            min: 0.0,
            max: f32::INFINITY,
        },
    );

    match intersection {
        Some(hit) => {
            let n = (hit.point - Vec3::new(0.0, 0.0, -1.0)).unit();
            ((n + 1.0) * 0.5).into()
        }
        None => {
            let direction = ray.direction.unit();
            let t = 0.5 * (direction.y + 1.0);
            Vec3::one().lerp(Vec3::new(0.5, 0.7, 1.0), t).into()
        }
    }
}
