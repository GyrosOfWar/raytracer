use std::fmt;
use std::ops::Mul;

use crate::vec::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Mat3 {
    data: [f32; 9],
}

impl Mat3 {
    pub const IDENTITY: Mat3 = Mat3 {
        data: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };
    pub const ZERO: Mat3 = Mat3 { data: [0.0; 9] };

    pub fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> Self {
        Self {
            data: [m00, m01, m02, m10, m11, m12, m20, m21, m22],
        }
    }

    pub fn from_cols(c0: Vec3, c1: Vec3, c2: Vec3) -> Self {
        Self {
            data: [c0.x, c1.x, c2.x, c0.y, c1.y, c2.y, c0.z, c1.z, c2.z],
        }
    }

    pub fn from_rows(r0: Vec3, r1: Vec3, r2: Vec3) -> Self {
        Self {
            data: [r0.x, r0.y, r0.z, r1.x, r1.y, r1.z, r2.x, r2.y, r2.z],
        }
    }

    pub fn from_diagonal(diagonal: Vec3) -> Self {
        Self {
            data: [
                diagonal.x, 0.0, 0.0, 0.0, diagonal.y, 0.0, 0.0, 0.0, diagonal.z,
            ],
        }
    }

    pub fn set(&mut self, i: usize, j: usize, value: f32) {
        self.data[i * 3 + j] = value;
    }

    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.data[i * 3 + j]
    }

    pub fn mat_mul(&self, rhs: &Mat3) -> Mat3 {
        let mut result = Mat3::ZERO;

        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result.data[i * 3 + j] += self.data[i * 3 + k] * rhs.data[k * 3 + j];
                }
            }
        }

        result
    }

    pub fn vec_mul(&self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self.data[0] * rhs.x + self.data[1] * rhs.y + self.data[2] * rhs.z,
            self.data[3] * rhs.x + self.data[4] * rhs.y + self.data[5] * rhs.z,
            self.data[6] * rhs.x + self.data[7] * rhs.y + self.data[8] * rhs.z,
        )
    }

    pub fn inverse(&self) -> Mat3 {
        let det = self.data[0] * (self.data[4] * self.data[8] - self.data[5] * self.data[7])
            - self.data[1] * (self.data[3] * self.data[8] - self.data[5] * self.data[6])
            + self.data[2] * (self.data[3] * self.data[7] - self.data[4] * self.data[6]);

        assert!(det != 0.0, "Matrix is not invertible");

        let inv_det = 1.0 / det;

        Mat3::new(
            (self.data[4] * self.data[8] - self.data[5] * self.data[7]) * inv_det,
            (self.data[2] * self.data[7] - self.data[1] * self.data[8]) * inv_det,
            (self.data[1] * self.data[5] - self.data[2] * self.data[4]) * inv_det,
            (self.data[5] * self.data[6] - self.data[3] * self.data[8]) * inv_det,
            (self.data[0] * self.data[8] - self.data[2] * self.data[6]) * inv_det,
            (self.data[2] * self.data[3] - self.data[0] * self.data[5]) * inv_det,
            (self.data[3] * self.data[7] - self.data[4] * self.data[6]) * inv_det,
            (self.data[1] * self.data[6] - self.data[0] * self.data[7]) * inv_det,
            (self.data[0] * self.data[4] - self.data[1] * self.data[3]) * inv_det,
        )
    }

    pub fn transpose(&self) -> Mat3 {
        Mat3::new(
            self.data[0],
            self.data[3],
            self.data[6],
            self.data[1],
            self.data[4],
            self.data[7],
            self.data[2],
            self.data[5],
            self.data[8],
        )
    }
}

impl fmt::Display for Mat3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[[{}, {}, {}], [{}, {}, {}], [{}, {}, {}]]",
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
            self.data[8]
        )
    }
}

impl Mul for Mat3 {
    type Output = Mat3;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mat_mul(&rhs)
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        self.vec_mul(rhs)
    }
}
