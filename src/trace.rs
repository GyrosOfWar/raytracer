use std::sync::Arc;

use crate::{
    bvh::{Aabb, BvhNode},
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};
use enum_dispatch::enum_dispatch;

#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl Default for Range {
    fn default() -> Self {
        Self { min: 0.0, max: 0.0 }
    }
}

impl Range {
    pub fn new(min: f32, max: f32) -> Self {
        Range { min, max }
    }

    pub fn from_ranges(a: Range, b: Range) -> Self {
        Range {
            min: if a.min <= b.min { a.min } else { b.min },
            max: if a.max >= b.max { a.max } else { b.max },
        }
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

    pub fn expand(&self, delta: f32) -> Self {
        let padding = delta / 2.0;
        Range {
            min: self.min - padding,
            max: self.max + padding,
        }
    }
}

#[derive(Debug)]
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

    fn bounding_box(&self) -> Aabb;
}

#[enum_dispatch(Hittable)]
#[derive(Debug)]
pub enum Object {
    Sphere(Sphere),
    BvhNode(BvhNode),
}

#[derive(Debug)]
pub struct World {
    objects: Vec<Arc<Object>>,
    bounding_box: Aabb,
}

fn make_bounding_box(objects: &[Object]) -> Aabb {
    let mut bbox = Aabb::default();
    for object in objects {
        bbox = Aabb::from_boxes(bbox, object.bounding_box());
    }
    bbox
}

impl World {
    pub fn new(objects: Vec<Object>) -> Self {
        Self {
            bounding_box: make_bounding_box(&objects),
            objects: objects.into_iter().map(Arc::new).collect(),
        }
    }

    pub fn objects(&self) -> &[Arc<Object>] {
        &self.objects
    }

    pub fn objects_mut(&mut self) -> &mut [Arc<Object>] {
        &mut self.objects
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

    fn bounding_box(&self) -> Aabb {
        self.bounding_box
    }
}

#[derive(Debug)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    pub material: Arc<Material>,
    bounding_box: Aabb,
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32, material: Arc<Material>) -> Self {
        let radius_vec = Vec3::new(radius, radius, radius);
        let bounding_box = Aabb::from_points(center - radius_vec, center + radius_vec);

        Sphere {
            center,
            radius,
            material,
            bounding_box,
        }
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

    fn bounding_box(&self) -> Aabb {
        self.bounding_box
    }
}

#[cfg(test)]
mod tests {
    use crate::{material::metal, trace::Hittable, vec3::Point3};

    use super::Sphere;

    #[test]
    fn get_sphere_bbox() {
        let material = metal(Point3::new(0.5, 0.1, 0.7), 0.2);
        let sphere = Sphere::new(Point3::new(0.0, 0.0, 0.0), 2.0, material);
        let bbox = sphere.bounding_box();
        assert_eq!(bbox.x.min, -2.0);
        assert_eq!(bbox.y.min, -2.0);
        assert_eq!(bbox.z.min, -2.0);
        assert_eq!(bbox.x.max, 2.0);
        assert_eq!(bbox.y.max, 2.0);
        assert_eq!(bbox.z.max, 2.0);
    }
}
