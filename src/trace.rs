use std::sync::Arc;

use crate::{
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};
use enum_dispatch::enum_dispatch;

pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl Range {
    pub fn new(min: f32, max: f32) -> Self {
        Range { min, max }
    }

    pub fn contains(&self, value: f32) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn surrounds(&self, value: f32) -> bool {
        self.min < value && value < self.max
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn clamp(&self, x: f32) -> f32 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }
}

pub struct HitRecord {
    pub point: Point3<f32>,
    pub normal: Vec3<f32>,
    pub distance: f32,
    pub front_facing: bool,
    pub material: Arc<Material>,
}

impl HitRecord {
    pub fn new(
        ray: &Ray<f32>,
        outward_normal: Vec3<f32>,
        point: Point3<f32>,
        distance: f32,
        material: Arc<Material>,
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
            material,
        }
    }
}

#[enum_dispatch]
pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord>;
}

#[derive(Debug)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    pub material: Arc<Material>,
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
            ))
        }
    }
}
