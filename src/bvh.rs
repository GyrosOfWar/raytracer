use std::{cmp::Reverse, sync::Arc, time::Instant};

use ordered_float::OrderedFloat;

use crate::{
    aabb::Aabb,
    object::{HitRecord, Hittable, Object, World},
    range::Range,
    ray::Ray,
    vec3::Axis,
};

#[derive(Debug)]
pub struct BvhNode {
    left: Arc<Object>,
    right: Arc<Object>,
    bbox: Aabb,
}

impl BvhNode {
    pub fn new(left: Arc<Object>, right: Arc<Object>) -> Self {
        Self {
            bbox: Aabb::from_boxes(left.bounding_box(), right.bounding_box()),
            left,
            right,
        }
    }

    pub fn from_world(world: &mut World) -> Self {
        let start = Instant::now();
        let objects = world.objects_mut();
        let root = BvhNode::from_objects(objects);
        println!("building BVH took {:?}", start.elapsed());
        root
    }

    fn from_objects(objects: &mut [Arc<Object>]) -> Self {
        let axis = Axis::random();
        let len = objects.len();

        match len {
            1 => BvhNode::new(objects[0].clone(), objects[0].clone()),
            2 => BvhNode::new(objects[0].clone(), objects[1].clone()),
            _ => {
                objects
                    .sort_by_key(|o| Reverse(OrderedFloat(o.bounding_box().interval_at(axis).min)));

                let mid = len / 2;
                let left = BvhNode::from_objects(&mut objects[0..mid]);
                let right = BvhNode::from_objects(&mut objects[mid..]);

                BvhNode::new(
                    Arc::new(Object::BvhNode(left)),
                    Arc::new(Object::BvhNode(right)),
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
        let hit_right = self.right.hit(
            ray,
            Range::new(
                hit_range.min,
                hit_left
                    .as_ref()
                    .map(|h| h.distance)
                    .unwrap_or(hit_range.max),
            ),
        );

        hit_left.or(hit_right)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
