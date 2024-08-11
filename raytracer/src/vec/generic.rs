use std::ops::Mul;

use num_traits::Float;

use crate::impl_generic_binary_op;

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

impl_generic_binary_op!(Add : add => (lhs: Vec3<T>, rhs: Vec3<T>) -> Vec3<T> {
    Vec3::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
        lhs.z + rhs.z,
    )
});

impl_generic_binary_op!(Mul : mul => (lhs: Vec3<T>, rhs: Vec3<T>) -> Vec3<T> {
    Vec3::new(
        lhs.x * rhs.x,
        lhs.y * rhs.y,
        lhs.z * rhs.z,
    )
});

impl_generic_binary_op!(Mul : mul => (lhs: Vec3<T>, rhs: T) -> Vec3<T> {
    Vec3::new(
        lhs.x * rhs,
        lhs.y * rhs,
        lhs.z * rhs,
    )
});

pub type Vec3f = Vec3<f32>;
