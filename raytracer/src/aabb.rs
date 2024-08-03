use crate::range::Range;
use crate::ray::Ray;
use crate::vec::{Axis, Point3};

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub x: Range,
    pub y: Range,
    pub z: Range,
}

impl Aabb {
    pub const EMPTY: Aabb = Aabb {
        x: Range::EMPTY,
        y: Range::EMPTY,
        z: Range::EMPTY,
    };
    pub const UNIVERSE: Aabb = Aabb {
        x: Range::UNIVERSE,
        y: Range::UNIVERSE,
        z: Range::UNIVERSE,
    };

    pub fn new(x: Range, y: Range, z: Range) -> Self {
        Aabb { x, y, z }.pad_to_minimums()
    }

    pub fn from_points(a: Point3, b: Point3) -> Self {
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
        let delta = 0.001f32;
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

    pub fn hit(&self, ray: &Ray, mut hit_range: Range) -> bool {
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

    pub fn longest_axis(&self) -> Axis {
        if self.x.size() > self.z.size() {
            Axis::X
        } else if self.y.size() > self.z.size() {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    pub fn contains(&self, rhs: &Aabb) -> bool {
        self.x.contains_range(&rhs.x)
            && self.y.contains_range(&rhs.y)
            && self.z.contains_range(&rhs.z)
    }
}
