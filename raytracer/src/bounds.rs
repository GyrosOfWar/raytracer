use std::mem;
use std::ops::{Add, Div, Mul, Neg, Sub};

use num_traits::{One, Zero};

use crate::math::{
    add_round_down, add_round_up, div_round_down, div_round_up, mul_round_down, mul_round_up,
    sub_round_down, sub_round_up,
};
use crate::range::Range;
use crate::ray::Ray;
use crate::util::{max_value, min_value};
use crate::vec::{Axis, IVec2, Point3, Vec2};

pub struct Bounds2f {
    p_min: Vec2,
    p_max: Vec2,
}

impl Bounds2f {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Bounds2f {
            p_min: a.min(b),
            p_max: a.max(b),
        }
    }

    pub fn p_min(&self) -> Vec2 {
        self.p_min
    }

    pub fn p_max(&self) -> Vec2 {
        self.p_max
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bounds2i {
    p_min: IVec2,
    p_max: IVec2,
}

impl Bounds2i {
    pub fn new(a: IVec2, b: IVec2) -> Self {
        Bounds2i {
            p_min: a.min(b),
            p_max: a.max(b),
        }
    }

    pub fn p_min(&self) -> IVec2 {
        self.p_min
    }

    pub fn p_max(&self) -> IVec2 {
        self.p_max
    }

    pub fn area(&self) -> i32 {
        let d = self.p_max - self.p_min;
        d.x * d.y
    }

    pub fn x_extent(&self) -> i32 {
        self.p_max.x - self.p_min.x
    }

    pub fn y_extent(&self) -> i32 {
        self.p_max.y - self.p_min.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds3 {
    pub x: Range,
    pub y: Range,
    pub z: Range,
}

impl Bounds3 {
    pub const EMPTY: Bounds3 = Bounds3 {
        x: Range::EMPTY,
        y: Range::EMPTY,
        z: Range::EMPTY,
    };
    pub const UNIVERSE: Bounds3 = Bounds3 {
        x: Range::UNIVERSE,
        y: Range::UNIVERSE,
        z: Range::UNIVERSE,
    };

    pub fn new(x: Range, y: Range, z: Range) -> Self {
        Bounds3 { x, y, z }.pad_to_minimums()
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

        Bounds3::new(x, y, z)
    }

    pub fn from_boxes(box0: Bounds3, box1: Bounds3) -> Self {
        Bounds3 {
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

    fn pad_to_minimums(mut self) -> Bounds3 {
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

    pub fn contains(&self, rhs: &Bounds3) -> bool {
        self.x.contains_range(&rhs.x)
            && self.y.contains_range(&rhs.y)
            && self.z.contains_range(&rhs.z)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Interval {
    low: f32,
    high: f32,
}

impl Interval {
    pub fn new(low: f32, high: f32) -> Self {
        assert!(!low.is_nan(), "low bound must not be NaN");
        assert!(!high.is_nan(), "high bound must not be NaN");

        Interval { low, high }
    }

    pub fn from_value(v: f32) -> Self {
        Interval::new(v, v)
    }

    pub fn from_value_and_error(v: f32, err: f32) -> Self {
        if err == 0.0 {
            Self::from_value(v)
        } else {
            Interval::new(sub_round_down(v, err), add_round_up(v, err))
        }
    }

    pub fn upper_bound(&self) -> f32 {
        self.high
    }

    pub fn lower_bound(&self) -> f32 {
        self.low
    }

    pub fn mid_point(&self) -> f32 {
        (self.high + self.low) / 2.0
    }

    pub fn width(&self) -> f32 {
        self.high - self.low
    }

    pub fn is_exactly_eq(&self, f: f32) -> bool {
        self.high == f && self.low == f
    }

    pub fn contains(&self, f: f32) -> bool {
        (self.low..self.high).contains(&f)
    }

    pub fn contains_range(&self, rhs: Interval) -> bool {
        self.low <= rhs.low && self.high >= rhs.high
    }

    pub fn is_finite(&self) -> bool {
        self.low.is_finite() && self.high.is_finite()
    }

    pub fn is_infinite(&self) -> bool {
        self.low.is_infinite() || self.high.is_infinite()
    }

    pub fn square(&self) -> Interval {
        let mut a_low = self.low.abs();
        let mut a_high = self.high.abs();
        if a_low > a_high {
            mem::swap(&mut a_low, &mut a_high);
        }

        if self.contains(0.0) {
            Interval::new(0.0, mul_round_up(a_high, a_high))
        } else {
            Interval::new(mul_round_down(a_low, a_low), mul_round_up(a_high, a_high))
        }
    }
}

impl PartialEq<f32> for Interval {
    fn eq(&self, other: &f32) -> bool {
        self.is_exactly_eq(*other)
    }
}

impl PartialEq<Interval> for Interval {
    fn eq(&self, other: &Interval) -> bool {
        self.low == other.low && self.high == other.high
    }
}

impl One for Interval {
    fn one() -> Self {
        Interval::from_value(1.0)
    }
}

impl Zero for Interval {
    fn zero() -> Self {
        Interval::from_value(0.0)
    }

    fn is_zero(&self) -> bool {
        self.is_exactly_eq(0.0)
    }

    fn set_zero(&mut self) {
        self.high = 0.0;
        self.low = 0.0;
    }
}

impl From<f32> for Interval {
    fn from(value: f32) -> Self {
        Interval::from_value(value)
    }
}

impl From<Interval> for f32 {
    fn from(value: Interval) -> Self {
        value.mid_point()
    }
}

impl Neg for Interval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Interval {
            high: -self.high,
            low: -self.low,
        }
    }
}

impl Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            low: add_round_down(self.low, rhs.low),
            high: add_round_up(self.high, rhs.high),
        }
    }
}

impl Sub for Interval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            low: sub_round_down(self.low, rhs.low),
            high: sub_round_up(self.high, rhs.high),
        }
    }
}

impl Div for Interval {
    type Output = Self;

    fn div(self, i: Self) -> Self::Output {
        if i.contains(0.0) {
            Interval::new(f32::NEG_INFINITY, f32::INFINITY)
        } else {
            let low = [
                div_round_down(self.low, i.low),
                div_round_down(self.high, i.low),
                div_round_down(self.low, i.high),
                div_round_down(self.high, i.high),
            ];
            let high = [
                div_round_up(self.low, i.low),
                div_round_up(self.high, i.low),
                div_round_up(self.low, i.high),
                div_round_up(self.high, i.high),
            ];

            Interval::new(min_value(&low), max_value(&high))
        }
    }
}

impl Mul for Interval {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let lp = [
            mul_round_down(self.low, rhs.low),
            mul_round_down(self.high, rhs.low),
            mul_round_down(self.low, rhs.high),
            mul_round_down(self.high, rhs.high),
        ];
        let hp = [
            mul_round_up(self.low, rhs.low),
            mul_round_up(self.high, rhs.low),
            mul_round_up(self.low, rhs.high),
            mul_round_up(self.high, rhs.high),
        ];

        Interval {
            low: min_value(&lp),
            high: max_value(&hp),
        }
    }
}
