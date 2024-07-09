use crate::{aabb::Aabb, range::Range, ray::Ray};

use super::{HitRecord, Hittable, Object};

#[derive(Debug)]
pub struct World {
    objects: Vec<Object>,
    bounding_box: Aabb,
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
}
