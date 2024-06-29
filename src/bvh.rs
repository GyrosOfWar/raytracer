use std::{cmp::Reverse, sync::Arc, time::Instant};

use ordered_float::OrderedFloat;

use crate::{
    aabb::Aabb,
    object::{get_id, HitRecord, Hittable, Object, World},
    range::Range,
    ray::Ray,
};

#[derive(Debug)]
pub struct BvhNode {
    left: Arc<Object>,
    right: Arc<Object>,
    bbox: Aabb,
    id: u64,
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

    pub fn from_world(world: &mut World) -> Self {
        let start = Instant::now();
        let objects = world.objects_mut();
        let root = BvhNode::from_objects(objects);
        println!("building BVH took {:?}", start.elapsed());
        root
    }

    fn from_objects(objects: &mut [Arc<Object>]) -> Self {
        let len = objects.len();

        match len {
            1 => BvhNode::new(
                objects[0].clone(),
                objects[0].clone(),
                objects[0].bounding_box(),
            ),
            2 => BvhNode::new(
                objects[0].clone(),
                objects[1].clone(),
                Aabb::from_boxes(objects[0].bounding_box(), objects[1].bounding_box()),
            ),
            _ => {
                let mut bbox = Aabb::EMPTY;
                for object in objects.iter() {
                    let bbox2 = object.bounding_box();
                    bbox = Aabb::from_boxes(bbox, bbox2);
                }
                let axis = bbox.longest_axis();

                bbox.assert_not_infinite();

                objects
                    .sort_by_key(|o| Reverse(OrderedFloat(o.bounding_box().interval_at(axis).min)));

                let mid = len / 2;
                let left = BvhNode::from_objects(&mut objects[0..mid]);
                let right = BvhNode::from_objects(&mut objects[mid..]);

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

    fn id(&self) -> u64 {
        self.id
    }
}

fn indent(level: usize) -> String {
    (0..(level * 2)).map(|_| " ").collect()
}

fn bbox_to_string(bbox: &Aabb) -> String {
    format!(
        "x = {:.2} {:.2}, y = {:.2} {:.2}, z={:.2} {:.2}",
        bbox.x.min, bbox.x.max, bbox.y.min, bbox.y.max, bbox.z.min, bbox.z.max,
    )
}

pub fn print_tree(object: Arc<Object>, level: usize) {
    let indent = indent(level);
    match object.as_ref() {
        Object::Sphere(s) => {
            println!(
                "{indent}- Sphere (id = {}, bbox = {}) ",
                s.id(),
                bbox_to_string(&s.bounding_box())
            );
        }
        Object::BvhNode(node) => {
            println!(
                "{indent}- Node (id = {}, bbox = {})",
                node.id(),
                bbox_to_string(&node.bounding_box())
            );
            print_tree(node.left.clone(), level + 1);
            print_tree(node.right.clone(), level + 1);
        }
    }
}
