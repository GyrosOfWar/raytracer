use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use num_traits::{Float, One, Zero};

#[derive(Debug, Clone, Copy)]
pub struct Vec3<T: Float> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub fn lerp<T: Float>(a: T, b: T, t: T) -> T {
    (T::one() - t) * a + t * b
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

    pub fn dot(self, rhs: Self) -> T {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn unit(self) -> Self {
        let len = self.length();
        self / len
    }

    pub fn lerp(&self, rhs: Self, t: T) -> Self {
        Vec3 {
            x: lerp(self.x, rhs.x, t),
            y: lerp(self.y, rhs.y, t),
            z: lerp(self.z, rhs.z, t),
        }
    }
}

impl Vec3<f32> {
    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        self.x.abs() < s && self.y.abs() < s && self.z.abs() < s
    }
}

pub fn reflect(v: Vec3<f32>, n: Vec3<f32>) -> Vec3<f32> {
    return v - n * Vec3::dot(v, n) * 2.0;
}

pub fn refract(uv: Vec3<f32>, n: Vec3<f32>, etai_over_etat: f32) -> Vec3<f32> {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parallel = n * -(1.0 - r_out_perp.length_squared()).abs().sqrt();
    r_out_perp + r_out_parallel
}

impl<T: Float> Neg for Vec3<T> {
    type Output = Vec3<T>;

    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
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

impl<T: Float + AddAssign> AddAssign for Vec3<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
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

impl<T: Float> One for Vec3<T> {
    fn one() -> Self {
        Vec3 {
            x: T::one(),
            y: T::one(),
            z: T::one(),
        }
    }
}

pub mod random {
    use crate::helpers::{random, random_range};

    use super::Vec3;

    pub fn gen() -> Vec3<f32> {
        Vec3::new(random(), random(), random())
    }

    pub fn gen_range(min: f32, max: f32) -> Vec3<f32> {
        Vec3::new(
            random_range(min, max),
            random_range(min, max),
            random_range(min, max),
        )
    }

    pub fn gen_unit_sphere() -> Vec3<f32> {
        loop {
            let p = gen_range(-1.0, 1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn gen_unit_disk() -> Vec3<f32> {
        loop {
            let p = Vec3::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0), 0.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn gen_unit_vector() -> Vec3<f32> {
        gen_unit_sphere().unit()
    }

    pub fn gen_on_hemisphere(normal: Vec3<f32>) -> Vec3<f32> {
        let on_unit_sphere = gen_unit_vector();
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }
}
