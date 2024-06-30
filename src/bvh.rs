use std::{sync::Arc, time::Instant};

use crate::{
    aabb::Aabb,
    object::{get_id, HitRecord, Hittable, Object},
    range::Range,
    ray::Ray,
};

#[derive(Debug)]
pub struct BvhNode {
    left: Arc<Object>,
    right: Arc<Object>,
    bbox: Aabb,
    id: u32,
}

impl BvhNode {
    pub fn new(left: Arc<Object>, right: Arc<Object>, bbox: Aabb) -> Self {
        let id = get_id();

        Self {
            bbox,
            left,
            right,
            id,
        }
    }

    pub fn from_world(objects: Vec<Object>) -> Self {
        let start = Instant::now();
        let root = BvhNode::from_objects(objects);
        println!("building BVH took {:?}", start.elapsed());
        root
    }

    fn from_objects(mut objects: Vec<Object>) -> Self {
        let len = objects.len();

        let mut bbox = Aabb::EMPTY;
        for object in objects.iter() {
            let bbox2 = object.bounding_box();
            bbox = Aabb::from_boxes(bbox, bbox2);
        }
        let axis = bbox.longest_axis();
        bbox.assert_not_infinite();

        match len {
            1 => {
                let object = Arc::new(objects.remove(0));
                BvhNode::new(object.clone(), object.clone(), bbox)
            }
            2 => {
                let left = Arc::new(objects.remove(0));
                let right = Arc::new(objects.remove(0));

                BvhNode::new(left, right, bbox)
            }
            _ => {
                objects.sort_by(|a, b| {
                    let r1 = a.bounding_box().interval_at(axis);
                    let r2 = b.bounding_box().interval_at(axis);
                    r2.min.partial_cmp(&r1.min).unwrap()
                });

                let mid = len / 2;
                let left = objects.drain(0..mid).collect();
                let right = objects;

                let left = BvhNode::from_objects(left);
                let right = BvhNode::from_objects(right);

                BvhNode::new(
                    Arc::new(Object::BvhNode(left)),
                    Arc::new(Object::BvhNode(right)),
                    bbox,
                )
            }
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        if !self.bbox.hit(ray, hit_range) {
            return None;
        }

        let hit_left = self.left.hit(ray, hit_range);
        let range = Range::new(
            hit_range.min,
            hit_left
                .as_ref()
                .map(|h| h.distance)
                .unwrap_or(hit_range.max),
        );

        let hit_right = self.right.hit(ray, range);

        hit_right.or(hit_left)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn id(&self) -> u32 {
        self.id
    }
}
