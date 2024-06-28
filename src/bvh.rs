use std::sync::Arc;

use crate::{
    ray::Ray,
    trace::{HitRecord, Hittable, Object, Range, World},
    vec3::{Axis, Point3},
};

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub x: Range,
    pub y: Range,
    pub z: Range,
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
        }
    }
}

impl Aabb {
    pub fn new(x: Range, y: Range, z: Range) -> Self {
        Aabb { x, y, z }.pad_to_minimums()
    }

    pub fn from_points(a: Point3<f32>, b: Point3<f32>) -> Self {
        let x = if a.x <= b.x {
            Range::new(a.x, b.x)
        } else {
            Range::new(b.x, a.x)
        };

        let y = if a.y <= b.y {
            Range::new(a.y, b.y)
        } else {
            Range::new(b.y, a.y)
        };

        let z = if a.z <= b.z {
            Range::new(a.z, b.z)
        } else {
            Range::new(b.z, a.z)
        };

        Aabb::new(x, y, z)
    }

    pub fn from_boxes(box0: Aabb, box1: Aabb) -> Self {
        Aabb {
            x: Range::from_ranges(box0.x, box1.x),
            y: Range::from_ranges(box0.y, box1.y),
            z: Range::from_ranges(box0.z, box1.z),
        }
        .pad_to_minimums()
    }

    pub fn interval_at(&self, axis: Axis) -> Range {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    fn pad_to_minimums(mut self) -> Aabb {
        let delta = 0.0001f32;
        if self.x.size() < delta {
            self.x = self.x.expand(delta);
        }

        if self.y.size() < delta {
            self.y = self.y.expand(delta);
        }

        if self.z.size() < delta {
            self.z = self.z.expand(delta);
        }

        self
    }

    pub fn hit(&self, ray: &Ray<f32>, mut hit_range: Range) -> bool {
        let ray_origin = ray.origin;
        let ray_direction = ray.direction;

        for axis in &[Axis::X, Axis::Y, Axis::Z] {
            let ax = self.interval_at(*axis);
            let ad_inv = 1.0 / ray_direction.at(*axis);
            let t0 = (ax.min - ray_origin.at(*axis)) * ad_inv;
            let t1 = (ax.max - ray_origin.at(*axis)) * ad_inv;

            if t0 < t1 {
                if t0 > hit_range.min {
                    hit_range.min = t0;
                }
                if t1 < hit_range.max {
                    hit_range.max = t1;
                }
            } else {
                if t1 > hit_range.min {
                    hit_range.min = t1;
                }
                if t0 < hit_range.max {
                    hit_range.max = t0;
                }
            }

            if hit_range.max <= hit_range.min {
                return false;
            }
        }

        true
    }
}

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
        let objects = world.objects_mut();
        BvhNode::from_objects(objects, 0, objects.len())
    }

    pub fn from_objects(objects: &mut [Arc<Object>], start: usize, end: usize) -> Self {
        let axis = Axis::random();
        let len = end - start;

        match len {
            1 => BvhNode::new(objects[start].clone(), objects[start].clone()),
            2 => BvhNode::new(objects[start].clone(), objects[start + 1].clone()),
            _ => {
                objects[start..end].sort_by(|a, b| {
                    let a_axis_interval = a.bounding_box().interval_at(axis);
                    let b_axis_interval = b.bounding_box().interval_at(axis);

                    a_axis_interval
                        .min
                        .partial_cmp(&b_axis_interval.min)
                        .expect("no NaNs allowed")
                });

                let mid = start + len / 2;
                let left = BvhNode::from_objects(objects, start, mid);
                let right = BvhNode::from_objects(objects, mid, end);

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
