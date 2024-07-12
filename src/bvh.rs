use std::cmp::Reverse;
use std::mem::MaybeUninit;
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

    pub fn is_valid(&self, nodes: &[FlatBvhNode], count: usize) -> bool {
        match self {
            FlatBvhNode::Leaf { .. } => true,
            FlatBvhNode::Interior { left, right, .. } => {
                let left = left
                    .and_then(|idx| nodes.get(idx))
                    .map(|n| n.is_valid(nodes, count + 1))
                    .unwrap_or(false);
                let right = right
                    .and_then(|idx| nodes.get(idx))
                    .map(|n| n.is_valid(nodes, count + 1))
                    .unwrap_or(false);

                left && right
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

fn handle_node(
    node: Option<Box<Object>>,
    node_list: &mut Vec<FlatBvhNode>,
    current_index: &mut usize,
) -> Option<usize> {
    if let Some(node) = node {
        *current_index += 1;
        match *node {
            Object::BvhNode(node) => {
                info!("handling interior node");
                let bbox = node.bounding_box();
                let left = handle_node(node.left, node_list, current_index);
                let right = handle_node(node.right, node_list, current_index);
                let node = FlatBvhNode::Interior { right, left, bbox };
                info!("pushing interior node {node:?}");
                node_list.push(node);
                Some(*current_index)
            }
            object => {
                info!("handling leaf object");
                node_list.push(FlatBvhNode::Leaf {
                    bbox: object.bounding_box(),
                    object,
                });
                Some(*current_index)
            }
        }
    } else {
        None
    }
}

fn flatten_tree(node: BvhNode, node_list: &mut Vec<MaybeUninit<FlatBvhNode>>, offset: &mut usize) {
    /*    LinearBVHNode *linearNode = &nodes[*offset];
    linearNode->bounds = node->bounds;
    int nodeOffset = (*offset)++;
    if (node->nPrimitives > 0) {
        CHECK(!node->children[0] && !node->children[1]);
        CHECK_LT(node->nPrimitives, 65536);
        linearNode->primitivesOffset = node->firstPrimOffset;
        linearNode->nPrimitives = node->nPrimitives;
    } else {
        // Create interior flattened BVH node
        linearNode->axis = node->splitAxis;
        linearNode->nPrimitives = 0;
        flattenBVH(node->children[0], offset);
        linearNode->secondChildOffset = flattenBVH(node->children[1], offset);
    }
    return nodeOffset;
     */

    let linear_node = &node_list[*offset];
    *offset += 1;
    let node_offset = *offset;
}

impl FlatBvhTree {
    pub fn from_tree(root: BvhNode) -> Self {
        let mut index = 0;
        let mut nodes = Vec::with_capacity(root.len());
        for _ in 0..root.len() {
            nodes.push(MaybeUninit::uninit());
        }
        flatten_tree(root, &mut nodes, &mut index);

        todo!()
        // FlatBvhTree { nodes }
    }

    pub fn is_valid(&self) -> bool {
        let root_node_is_interior = self.nodes.len() > 1 && self.nodes[0].is_interior();
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
}

// TODO refactor: turn into enum with interior and leaf nodes
// Leaf nodes contain the Box<Object> and the bounding box
// Interior nodes contain the left and right children (Box<BvhNode>) and the bounding box
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

    pub fn from_object(object: Object) -> Self {
        if let Object::World(world) = object {
            BvhNode::from(world.objects)
        } else {
            BvhNode::from(vec![object])
        }
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

    pub fn len(&self) -> usize {
        match (self.left.as_ref(), self.right.as_ref()) {
            (Some(left), Some(right)) => left.len() + right.len(),
            (Some(left), None) => left.len(),
            (None, Some(right)) => right.len(),
            (None, None) => 1,
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

#[cfg(test)]
mod tests {
    use tracing::debug;
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
