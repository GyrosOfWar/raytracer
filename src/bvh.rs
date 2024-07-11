use std::cmp::Reverse;
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
pub enum FlatBvhNode {
    Leaf {
        object: Object,
        bbox: Aabb,
    },
    Interior {
        left: Option<usize>,
        right: Option<usize>,
        bbox: Aabb,
    },
}

#[derive(Debug)]
pub struct FlatBvhTree {
    nodes: Vec<FlatBvhNode>,
}

fn handle_node(node: Option<Box<Object>>, current_index: &mut usize) -> Option<FlatBvhNode> {
    if let Some(node) = node {
        match *node {
            Object::BvhNode(node) => {
                let bbox = node.bounding_box();
                let left = handle_node(node.left, current_index);
                let left = left.map(|_| {
                    *current_index += 1;
                    *current_index
                });
                let right = handle_node(node.right, current_index);
                let right = right.map(|_| {
                    *current_index += 1;
                    *current_index
                });
                Some(FlatBvhNode::Interior { right, left, bbox })
            }
            object => {
                *current_index += 1;
                Some(FlatBvhNode::Leaf {
                    bbox: object.bounding_box(),
                    object,
                })
            }
        }
    } else {
        None
    }
}

fn flatten_tree(node: BvhNode, node_list: &mut Vec<FlatBvhNode>, current_index: &mut usize) {
    if let Some(node) = handle_node(node.left, current_index) {
        node_list.push(node);
    }

    if let Some(node) = handle_node(node.right, current_index) {
        node_list.push(node);
    }
}

impl FlatBvhTree {
    pub fn from_tree(root: BvhNode) -> Self {
        let mut index = 0;
        let mut nodes = vec![];
        flatten_tree(root, &mut nodes, &mut index);

        FlatBvhTree { nodes }
    }
}

impl Hittable for FlatBvhTree {
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord> {
        todo!()
    }

    fn bounding_box(&self) -> Aabb {
        self.nodes
            .get(0)
            .map(|node| match node {
                FlatBvhNode::Leaf { bbox, .. } => *bbox,
                FlatBvhNode::Interior { bbox, .. } => *bbox,
            })
            .unwrap_or(Aabb::EMPTY)
    }

    fn id(&self) -> u32 {
        0
    }
}

#[derive(Debug)]
pub struct BvhNode {
    left: Option<Box<Object>>,
    right: Option<Box<Object>>,
    bbox: Aabb,
    id: u32,
}

impl BvhNode {
    pub fn new(left: Option<Box<Object>>, right: Option<Box<Object>>, bbox: Aabb) -> Self {
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
                let object = objects.remove(0);
                BvhNode::new(Some(Box::new(object)), None, bbox)
            }
            2 => {
                let left = Box::new(objects.remove(0));
                let right = Box::new(objects.remove(0));

                BvhNode::new(Some(left), Some(right), bbox)
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
                    Some(Box::new(Object::BvhNode(left))),
                    Some(Box::new(Object::BvhNode(right))),
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

        match (self.left.as_ref(), self.right.as_ref()) {
            (Some(left), Some(right)) => {
                let hit_left = left.hit(ray, hit_range);
                let range = Range::new(
                    hit_range.min,
                    hit_left
                        .as_ref()
                        .map(|h| h.distance)
                        .unwrap_or(hit_range.max),
                );

                let hit_right = right.hit(ray, range);

                hit_right.or(hit_left)
            }
            (Some(left), None) => left.hit(ray, hit_range),
            (None, Some(right)) => right.hit(ray, hit_range),
            (None, None) => None,
        }
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
}
