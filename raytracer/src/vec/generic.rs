use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use num_traits::{Float, One, Zero};

use super::{Axis, Point3fi, VectorLike};
use crate::bounds::Interval;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T> Vec3<T>
where
    T: Copy,
{
    pub fn all(v: T) -> Self {
        Vec3::new(v, v, v)
    }

    pub fn get(&self, i: usize) -> T {
        assert!(i < 3, "index out of bounds");
        match i {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => unreachable!(),
        }
    }

    pub fn at(&self, axis: Axis) -> T {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }
}

impl<T> Vec3<T>
where
    T: Float,
{
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    pub fn length_squared(self) -> T {
        self.dot(self)
    }

    pub fn dot(&self, rhs: Self) -> T {
        T::mul_add(self.x, rhs.x, T::mul_add(self.y, rhs.y, self.z * rhs.z))
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub fn normalized(self) -> Self {
        self * self.length().recip()
    }

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
}

impl<T: Copy> VectorLike<3, T> for Vec3<T> {
    fn component(&self, index: usize) -> T {
        self.get(index)
    }

    fn data(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    fn from_data(data: [T; 3]) -> Self {
        Vec3::new(data[0], data[1], data[2])
    }
}

impl<T> One for Vec3<T>
where
    T: One + Copy,
{
    fn one() -> Self {
        Vec3::all(T::one())
    }
}

impl<T> Zero for Vec3<T>
where
    T: Zero + Copy,
{
    fn zero() -> Self {
        Vec3::all(T::zero())
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}

impl<T> Add for Vec3<T>
where
    T: Copy + Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T> Add<T> for Vec3<T>
where
    T: Copy + Add<Output = T>,
{
    type Output = Vec3<T>;

    fn add(self, rhs: T) -> Vec3<T> {
        Vec3::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl<T> Sub for Vec3<T>
where
    T: Copy + Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T> Sub for &Vec3<T>
where
    T: Copy + Sub<Output = T>,
{
    type Output = Vec3<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T> Sub<T> for Vec3<T>
where
    T: Copy + Sub<Output = T>,
{
    type Output = Vec3<T>;

    fn sub(self, rhs: T) -> Vec3<T> {
        Vec3::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl<T> Neg for Vec3<T>
where
    T: Copy + Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl<T> Mul for Vec3<T>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl<T> Mul<T> for Vec3<T>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: T) -> Vec3<T> {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<T> Mul<T> for &Vec3<T>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: T) -> Vec3<T> {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<T> Div for Vec3<T>
where
    T: Copy + Div<Output = T>,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl<T> Div<T> for Vec3<T>
where
    T: Copy + Div<Output = T>,
{
    type Output = Vec3<T>;

    fn div(self, rhs: T) -> Vec3<T> {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<T> Div<T> for &Vec3<T>
where
    T: Copy + Div<Output = T>,
{
    type Output = Vec3<T>;

    fn div(self, rhs: T) -> Vec3<T> {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Point3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy> VectorLike<3, T> for Point3<T> {
    fn component(&self, index: usize) -> T {
        self.get(index)
    }

    fn data(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    fn from_data(data: [T; 3]) -> Self {
        Point3::new(data[0], data[1], data[2])
    }
}

impl<T: Copy> Point3<T> {
    pub fn at(&self, axis: Axis) -> T {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    pub fn get(&self, index: usize) -> T {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("index out of bounds"),
        }
    }

    pub fn all(v: T) -> Self {
        Point3::new(v, v, v)
    }
}

impl<T: Zero + Copy> Point3<T> {
    pub fn zero() -> Self {
        Point3::all(T::zero())
    }
}

impl<T: One + Copy> Point3<T> {
    pub fn one() -> Self {
        Point3::all(T::one())
    }
}

impl<T> From<Vec3<T>> for Point3<T> {
    fn from(v: Vec3<T>) -> Self {
        Point3::new(v.x, v.y, v.z)
    }
}

impl<T> From<Point3<T>> for Vec3<T> {
    fn from(value: Point3<T>) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

impl<T> Add<T> for Point3<T>
where
    T: Copy + Add<Output = T>,
{
    type Output = Point3<T>;

    fn add(self, rhs: T) -> Self::Output {
        Point3::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl<T> Add<Vec3<T>> for Point3<T>
where
    T: Copy + Add<Output = T>,
{
    type Output = Point3<T>;

    fn add(self, rhs: Vec3<T>) -> Self::Output {
        Point3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T> Sub for Point3<T>
where
    T: Copy + Sub<Output = T>,
{
    type Output = Vec3<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T> Div<T> for Point3<T>
where
    T: Copy + Div<Output = T>,
{
    type Output = Point3<T>;

    fn div(self, rhs: T) -> Self::Output {
        Point3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<T> AddAssign<Vec3<T>> for Point3<T>
where
    T: Copy + AddAssign,
{
    fn add_assign(&mut self, rhs: Vec3<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Copy> Vec2<T> {
    pub fn all(v: T) -> Self {
        Vec2::new(v, v)
    }

    pub fn get(&self, i: usize) -> T {
        assert!(i < 2, "index out of bounds");
        match i {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }
}

impl Point3<Interval> {
    pub fn from_value_and_error(p: Point3<f32>, e: Vec3<f32>) -> Self {
        Point3 {
            x: Interval::from_value_and_error(p.x, e.x),
            y: Interval::from_value_and_error(p.y, e.y),
            z: Interval::from_value_and_error(p.z, e.z),
        }
    }

    pub fn is_exact(&self) -> bool {
        self.x.width() == 0.0 && self.y.width() == 0.0 && self.z.width() == 0.0
    }

    pub fn error(&self) -> Vec3f {
        Vec3::new(
            self.x.width() / 2.0,
            self.y.width() / 2.0,
            self.z.width() / 2.0,
        )
    }
}

impl From<Point3f> for Point3fi {
    fn from(value: Point3f) -> Self {
        Point3 {
            x: value.x.into(),
            y: value.y.into(),
            z: value.z.into(),
        }
    }
}

impl From<Point3fi> for Point3f {
    fn from(value: Point3fi) -> Self {
        Point3 {
            x: value.x.into(),
            y: value.y.into(),
            z: value.z.into(),
        }
    }
}

impl From<Vec3<Interval>> for Vec3f {
    fn from(value: Vec3<Interval>) -> Self {
        Vec3 {
            x: value.x.into(),
            y: value.y.into(),
            z: value.z.into(),
        }
    }
}

impl From<Vec3f> for Vec3<Interval> {
    fn from(value: Vec3f) -> Self {
        Vec3 {
            x: value.x.into(),
            y: value.y.into(),
            z: value.z.into(),
        }
    }
}

pub type Vec3f = Vec3<f32>;
pub type Point3f = Point3<f32>;
