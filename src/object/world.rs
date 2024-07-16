use super::{HitRecord, Hittable, Object};
use crate::aabb::Aabb;
use crate::range::Range;
use crate::ray::Ray;

#[derive(Debug)]
pub struct World {
    pub objects: Vec<Object>,
    pub bounding_box: Aabb,
}

impl World {
    pub fn new(objects: Vec<Object>) -> Self {
        Self {
            bounding_box: Aabb::from_objects(&objects),
            objects,
        }
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord> {
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

    fn id(&self) -> u32 {
        0
    }

    fn name(&self) -> &'static str {
        "World"
    }
}
