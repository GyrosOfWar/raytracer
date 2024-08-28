use std::ops::{Add, Mul};

use num_traits::{Float, One, Zero};

use crate::bounds::Interval;

#[derive(Debug, Clone, Copy)]
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

impl<T> Mul<T> for Vec3<T>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: T) -> Vec3<T> {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
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

impl<T> Mul for Vec3<T>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
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

#[derive(Debug)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Point3<T> {
    fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
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

pub type Vec3f = Vec3<f32>;
pub type Point3f = Point3<f32>;
pub type Point3fi = Point3<Interval>;
