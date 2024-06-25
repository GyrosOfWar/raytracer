use std::ops::{Add, Div, Mul, Sub};

use num_traits::{Float, Zero};

#[derive(Debug, Clone, Copy)]
pub struct Vec3<T: Float> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Point3<T> = Vec3<T>;

impl<T: Float> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Vec3 { x, y, z }
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, rhs: Self) -> T {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: Self) -> Self {
        Vec3 {
            x: self.y * rhs.z - self.y * rhs.z,
            y: self.x * rhs.z - self.x * rhs.z,
            z: self.x * rhs.y - self.x * rhs.y,
        }
    }

    pub fn unit(self) -> Self {
        let len = self.length();
        self / len
    }
}

impl<T: Float> Add for Vec3<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: Float> Add<T> for Vec3<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self {
        Vec3 {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

impl<T: Float> Sub for Vec3<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T: Float> Mul for Vec3<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T: Float> Mul<T> for Vec3<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T: Float> Div<T> for Vec3<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<T: Float> Zero for Vec3<T> {
    fn zero() -> Self {
        Vec3 {
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}
