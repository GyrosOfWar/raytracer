use std::ops::Mul;

use num_traits::Float;

#[derive(Debug, Clone, Copy)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T>
where
    T: Copy,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn all(v: T) -> Self {
        Vec3::new(v, v, v)
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

    pub fn get(&self, i: usize) -> T {
        assert!(i < 3, "index out of bounds");
        match i {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => unreachable!(),
        }
    }

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
}

impl<T> Mul for Vec3<T>
where
    T: Mul<Output = T>,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T, F: Float> Mul<F> for Vec3<T>
where
    T: Mul<F, Output = T>,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: F) -> Vec3<T> {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

pub type Vec3f = Vec3<f32>;
