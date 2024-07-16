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

pub enum BvhType {
    Flat,
    Tree,
}

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

impl FlatBvhNode {
    const EMPTY: FlatBvhNode = FlatBvhNode::Interior {
        left: None,
        right: None,
        bbox: Aabb::EMPTY,
    };

    pub fn bounding_box(&self) -> Aabb {
        match self {
            FlatBvhNode::Leaf { bbox, .. } => *bbox,
            FlatBvhNode::Interior { bbox, .. } => *bbox,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, FlatBvhNode::Leaf { .. })
    }

    pub fn is_interior(&self) -> bool {
        matches!(self, FlatBvhNode::Interior { .. })
    }

    pub fn set_indices(&mut self, l: usize, r: usize) {
        if let FlatBvhNode::Interior { left, right, .. } = self {
            left.replace(l);
            right.replace(r);
        }
    }

    pub fn is_valid(&self, nodes: &[FlatBvhNode], count: usize) -> bool {
        match self {
            FlatBvhNode::Leaf { .. } => true,
            FlatBvhNode::Interior { left, right, .. } => {
                let left_valid = left
                    .and_then(|idx| nodes.get(idx))
                    .map(|n| n.is_valid(nodes, count + 1))
                    .unwrap_or(false);
                info!("left {:?} valid: {}", left, left_valid);
                let right_valid = right
                    .and_then(|idx| nodes.get(idx))
                    .map(|n| n.is_valid(nodes, count + 1))
                    .unwrap_or(false);
                info!("right {:?} valid: {}", right, right_valid);

                left_valid && right_valid
            }
        }
    }

    pub fn hit(&self, ray: &Ray, hit_range: Range, nodes: &[FlatBvhNode]) -> Option<HitRecord> {
        match self {
            FlatBvhNode::Interior { left, right, bbox } => {
                if !bbox.hit(ray, hit_range) {
                    return None;
                }
                let hit_left = left.and_then(|idx| nodes[idx].hit(ray, hit_range, nodes));
                let range = Range::new(
                    hit_range.min,
                    hit_left
                        .as_ref()
                        .map(|h| h.distance)
                        .unwrap_or(hit_range.max),
                );
                let hit_right = right.and_then(|idx| nodes[idx].hit(ray, range, nodes));

                hit_right.or(hit_left)
            }

            FlatBvhNode::Leaf { object, bbox } => {
                if !bbox.hit(ray, hit_range) {
                    return None;
                }

                object.hit(ray, hit_range)
            }
        }
    }
}

#[derive(Debug)]
pub struct FlatBvhTree {
    nodes: Vec<FlatBvhNode>,
}

fn flatten_tree(node: BvhNode, node_list: &mut Vec<FlatBvhNode>, offset: &mut usize) -> usize {
    let node_offset = *offset;
    *offset += 1;
    match node {
        BvhNode::Interior {
            left, right, bbox, ..
        } => {
            // info!("interior: node offset: {node_offset}");
            node_list[node_offset] = FlatBvhNode::Interior {
                left: None,
                right: None,
                bbox: bbox,
            };
            let left_index = flatten_tree(*left, node_list, offset);
            let right_index = flatten_tree(*right, node_list, offset);

            node_list[node_offset].set_indices(left_index, right_index);
        }
        BvhNode::Leaf { object, bbox, .. } => {
            // info!("leaf: node offset: {node_offset}");
            node_list[node_offset] = FlatBvhNode::Leaf {
                object: *object,
                bbox: bbox,
            };
        }
    }

    *offset
}

impl FlatBvhTree {
    pub fn from_tree(root: BvhNode) -> Self {
        let len = root.len() * 2;
        let mut index = 0;
        let mut nodes = Vec::with_capacity(len);
        for _ in 0..len {
            nodes.push(FlatBvhNode::Interior {
                left: None,
                right: None,
                bbox: Aabb::EMPTY,
            });
        }
        flatten_tree(root, &mut nodes, &mut index);

        Self { nodes }
    }

    pub fn is_valid(&self) -> bool {
        let root_node_is_interior = self.nodes.len() > 1 && self.nodes[0].is_interior();
        info!("root node is interior: {root_node_is_interior}");
        root_node_is_interior && self.nodes[0].is_valid(&self.nodes, 0)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl Hittable for FlatBvhTree {
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord> {
        let root = &self.nodes[0];

        root.hit(ray, hit_range, &self.nodes)
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

    fn name(&self) -> &'static str {
        "FlatBvhTree"
    }
}

#[derive(Debug)]
pub enum BvhNode {
    Interior {
        left: Box<BvhNode>,
        right: Box<BvhNode>,
        id: u32,
        bbox: Aabb,
    },
    Leaf {
        object: Box<Object>,
        id: u32,
        bbox: Aabb,
    },
}

impl BvhNode {
    pub fn from(objects: Vec<Object>) -> Self {
        let start = Instant::now();
        let root = BvhNode::from_objects(objects);
        info!("building BVH took {:?}", start.elapsed());
        root
    }

    pub fn from_object(object: Object) -> Self {
        if let Object::World(world) = object {
            BvhNode::from(world.objects)
        } else {
            BvhNode::from(vec![object])
        }
    }

    fn bbox(&self) -> Aabb {
        match self {
            BvhNode::Interior { bbox, .. } => *bbox,
            BvhNode::Leaf { bbox, .. } => *bbox,
        }
    }

    fn id(&self) -> u32 {
        match self {
            BvhNode::Interior { id, .. } => *id,
            BvhNode::Leaf { id, .. } => *id,
        }
    }

    fn from_objects(mut objects: Vec<Object>) -> Self {
        let len = objects.len();

        let bbox = Aabb::from_objects(&objects);
        let axis = bbox.longest_axis();

        match len {
            1 => {
                let object = objects.remove(0);
                BvhNode::Leaf {
                    bbox: object.bounding_box(),
                    id: get_id(),
                    object: Box::new(object),
                }
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

                BvhNode::Interior {
                    left: Box::new(left),
                    right: Box::new(right),
                    id: get_id(),
                    bbox,
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            BvhNode::Interior { left, right, .. } => left.len() + right.len(),
            BvhNode::Leaf { .. } => 1,
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord> {
        if !self.bbox().hit(ray, hit_range) {
            return None;
        }

        match self {
            BvhNode::Interior { left, right, .. } => {
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
            BvhNode::Leaf { object, .. } => object.hit(ray, hit_range),
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox()
    }

    fn id(&self) -> u32 {
        self.id()
    }

    fn name(&self) -> &'static str {
        "BvhNode"
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::{BvhNode, FlatBvhTree};
    use crate::{scene, Result};

    #[test]
    #[traced_test]
    fn test_build_linear_bvh() -> Result<()> {
        let scene = scene::load_from_gltf("./assets/cornell.gltf")?;
        let node = BvhNode::from_object(scene.root_object);
        let tree = FlatBvhTree::from_tree(node);
        // debug!("{tree:#?}");
        assert!(tree.is_valid());

        Ok(())
    }
}
