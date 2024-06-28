use crate::{
    ray::Ray,
    trace::Range,
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
        Aabb { x, y, z }
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

        Aabb { x, y, z }
    }

    pub fn from_boxes(box0: Aabb, box1: Aabb) -> Self {
        Aabb {
            x: Range::from_ranges(box0.x, box1.x),
            y: Range::from_ranges(box0.y, box1.y),
            z: Range::from_ranges(box0.z, box1.z),
        }
    }

    fn interval_at(&self, axis: Axis) -> Range {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    // this probably isn't right
    pub fn hit(&self, ray: &Ray<f32>, mut interval: Range) -> bool {
        let ray_origin = ray.origin;
        let ray_direction = ray.direction;

        for axis in &[Axis::X, Axis::Y, Axis::Z] {
            let ax = self.interval_at(*axis);
            let ad_inv = 1.0 / ray_direction.at(*axis);
            let t0 = (ax.min - ray_origin.at(*axis)) * ad_inv;
            let t1 = (ax.max - ray_origin.at(*axis)) * ad_inv;

            if t0 < t1 {
                if t0 > interval.min {
                    interval.min = t0;
                }
                if t1 < interval.max {
                    interval.max = t1;
                }
            } else {
                if t1 > interval.min {
                    interval.min = t1;
                }
                if t0 < interval.max {
                    interval.max = t0;
                }
            }

            if interval.max <= interval.min {
                return false;
            }
        }

        true
    }
}