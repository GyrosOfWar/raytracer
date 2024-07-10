use std::cmp::Reverse;
use std::sync::Arc;
use std::time::Instant;

use ordered_float::OrderedFloat;
use rayon::prelude::*;
use tracing::info;

use crate::aabb::Aabb;
use crate::object::{get_id, HitRecord, Hittable, Object};
use crate::range::Range;
use crate::ray::Ray;

// TODOs
//  - Experiment with different heuristics for splitting the objects (look at pbrt)
//  - Store all nodes in a contiguous list instead of a pointer-y tree

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

    pub fn from(objects: Vec<Object>) -> Self {
        let start = Instant::now();
        let root = BvhNode::from_objects(objects);
        info!("building BVH took {:?}", start.elapsed());
        root
    }

    fn from_objects(mut objects: Vec<Object>) -> Self {
        let len = objects.len();

        let bbox = Aabb::from_objects(&objects);
        let axis = bbox.longest_axis();

        match len {
            1 => {
                let object = Arc::new(objects.remove(0));
                BvhNode::new(object.clone(), object, bbox)
            }
            2 => {
                let left = Arc::new(objects.remove(0));
                let right = Arc::new(objects.remove(0));

                BvhNode::new(left, right, bbox)
            }
            _ => {
                objects.par_sort_unstable_by_key(|o| {
                    Reverse(OrderedFloat(o.bounding_box().interval_at(axis).min))
                });

                let mid = len / 2;
                let left = objects.par_drain(0..mid).collect();
                let right = objects;

                let (left, right) = rayon::join(
                    || BvhNode::from_objects(left),
                    || BvhNode::from_objects(right),
                );

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
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord> {
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

#[allow(unused)]
pub mod debug {
    use std::sync::Arc;

    use tracing::error;

    use crate::aabb::Aabb;
    use crate::object::{Hittable, Object};

    fn indent(level: usize) -> String {
        (0..(level * 2)).map(|_| " ").collect()
    }

    fn bbox_to_string(bbox: &Aabb) -> String {
        format!(
            "x = {} {}, y = {} {}, z={} {}",
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
            Object::TriangleRef(t) => {
                println!(
                    "{indent}- TriangleRef (id = {}, bbox = {})",
                    t.id(),
                    bbox_to_string(&t.bounding_box())
                );
            }
            Object::World(_) => panic!("World should not be in the tree"),
        }
    }

    pub fn validate_tree(object: Arc<Object>) -> bool {
        // make sure the bounding box contains all the children
        let bbox = object.bounding_box();
        let mut valid = true;
        match object.as_ref() {
            Object::Sphere(s) => {
                if !bbox.contains(&s.bounding_box()) {
                    error!("Sphere {} not contained in parent", s.id());
                    valid = false;
                }
            }
            Object::BvhNode(node) => {
                if !bbox.contains(&node.bounding_box()) {
                    error!("Node {} not contained in parent", node.id());
                    valid = false;
                }
                valid &= validate_tree(node.left.clone());
                valid &= validate_tree(node.right.clone());
            }
            Object::TriangleRef(t) => {
                if !bbox.contains(&t.bounding_box()) {
                    error!("TriangleRef {} not contained in parent", t.id());
                    valid = false;
                }
            }
            Object::World(_) => panic!("World should not be in the tree"),
        }

        valid
    }
}
