use std::ops::Mul;

use super::{Point3, VectorLike};
use crate::vec::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Mat4 {
    data: [[f32; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        data: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
    pub const ZERO: Mat4 = Mat4 {
        data: [[0.0; 4]; 4],
    };

    pub fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m30: f32,
        m31: f32,
        m32: f32,
        m33: f32,
    ) -> Self {
        Self {
            data: [
                [m00, m01, m02, m03],
                [m10, m11, m12, m13],
                [m20, m21, m22, m23],
                [m30, m31, m32, m33],
            ],
        }
    }

    pub fn from_rows(r0: [f32; 4], r1: [f32; 4], r2: [f32; 4], r3: [f32; 4]) -> Self {
        Self {
            data: [r0, r1, r2, r3],
        }
    }

    pub fn inverse(&self) -> Mat4 {
        todo!()
    }

    pub fn from_translation(z: Vec3) -> Mat4 {
        Self {
            data: [
                [1.0, 0.0, 0.0, z.x],
                [0.0, 1.0, 0.0, z.y],
                [0.0, 0.0, 1.0, z.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_rotation_x(angle: f32) -> Mat4 {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, angle.cos(), -angle.sin(), 0.0],
                [0.0, angle.sin(), angle.cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_rotation_y(angle: f32) -> Mat4 {
        Self {
            data: [
                [angle.cos(), 0.0, angle.sin(), 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-angle.sin(), 0.0, angle.cos(), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_rotation_z(angle: f32) -> Mat4 {
        Self {
            data: [
                [angle.cos(), -angle.sin(), 0.0, 0.0],
                [angle.sin(), angle.cos(), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_scale(z: Vec3) -> Mat4 {
        Self {
            data: [
                [z.x, 0.0, 0.0, 0.0],
                [0.0, z.y, 0.0, 0.0],
                [0.0, 0.0, z.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn vec_mul<T>(&self, v: T) -> T
    where
        T: VectorLike<3, f32>,
    {
        let x = self.data[0][0] * v.component(0)
            + self.data[0][1] * v.component(1)
            + self.data[0][2] * v.component(2)
            + self.data[0][3];
        let y = self.data[1][0] * v.component(0)
            + self.data[1][1] * v.component(1)
            + self.data[1][2] * v.component(2)
            + self.data[1][3];
        let z = self.data[2][0] * v.component(0)
            + self.data[2][1] * v.component(1)
            + self.data[2][2] * v.component(2)
            + self.data[2][3];

        T::from_data([x, y, z])
    }

    pub fn set(&mut self, i: usize, j: usize, value: f32) {
        self.data[i][j] = value;
    }

    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.data[i][j]
    }

    pub fn mat_mul(&self, rhs: Mat4) -> Mat4 {
        let mut result = Mat4::ZERO;

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.set(
                        i,
                        j,
                        f32::mul_add(self.get(i, k), rhs.get(k, j), result.get(i, j)),
                    );
                }
            }
        }

        result
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mat_mul(rhs)
    }
}

impl Mul<Vec3> for Mat4 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        self.vec_mul(rhs)
    }
}

impl Mul<Point3> for Mat4 {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        self.vec_mul(rhs)
    }
}
