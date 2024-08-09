use std::fmt;
use std::ops::Mul;

use crate::vec::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Mat3 {
    data: [[f32; 3]; 3],
}

impl Mat3 {
    pub const IDENTITY: Mat3 = Mat3 {
        data: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };
    pub const ZERO: Mat3 = Mat3 {
        data: [[0.0; 3]; 3],
    };

    /// Create a new matrix.
    #[allow(clippy::too_many_arguments)]
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
            data: [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]],
        }
    }

    /// Create a matrix from column vectors.
    pub fn from_cols(c0: Vec3, c1: Vec3, c2: Vec3) -> Self {
        Self {
            data: [[c0.x, c1.x, c2.x], [c0.y, c1.y, c2.y], [c0.z, c1.z, c2.z]],
        }
    }

    /// Create a matrix from row vectors.
    pub fn from_rows(r0: Vec3, r1: Vec3, r2: Vec3) -> Self {
        Self {
            data: [[r0.x, r0.y, r0.z], [r1.x, r1.y, r1.z], [r2.x, r2.y, r2.z]],
        }
    }

    /// Create a diagonal matrix.
    pub fn from_diagonal(d: Vec3) -> Self {
        Self {
            data: [[d.x, 0.0, 0.0], [0.0, d.y, 0.0], [0.0, 0.0, d.z]],
        }
    }

    pub fn set(&mut self, i: usize, j: usize, value: f32) {
        self.data[i][j] = value;
    }

    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.data[i][j]
    }

    pub fn row(&self, i: usize) -> Vec3 {
        Vec3::new(self.data[i][0], self.data[i][1], self.data[i][2])
    }

    pub fn col(&self, j: usize) -> Vec3 {
        Vec3::new(self.data[0][j], self.data[1][j], self.data[2][j])
    }

    pub fn mat_mul(&self, rhs: &Mat3) -> Mat3 {
        let mut result = Mat3::ZERO;

        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
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

    pub fn vec_mul(&self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self.row(0).dot(rhs),
            self.row(1).dot(rhs),
            self.row(2).dot(rhs),
        )
    }

    pub fn try_inverse(&self) -> Option<Mat3> {
        let det = self.data[0][0]
            * (self.data[1][1] * self.data[2][2] - self.data[1][2] * self.data[2][1])
            - self.data[0][1]
                * (self.data[1][0] * self.data[2][2] - self.data[1][2] * self.data[2][0])
            + self.data[0][2]
                * (self.data[1][0] * self.data[2][1] - self.data[1][1] * self.data[2][0]);

        if det == 0.0 {
            None
        } else {
            let inv_det = 1.0 / det;

            Some(Mat3::new(
                (self.data[1][1] * self.data[2][2] - self.data[1][2] * self.data[2][1]) * inv_det,
                (self.data[0][2] * self.data[2][1] - self.data[0][1] * self.data[2][2]) * inv_det,
                (self.data[0][1] * self.data[1][2] - self.data[0][2] * self.data[1][1]) * inv_det,
                (self.data[1][2] * self.data[2][0] - self.data[1][0] * self.data[2][2]) * inv_det,
                (self.data[0][0] * self.data[2][2] - self.data[0][2] * self.data[2][0]) * inv_det,
                (self.data[0][2] * self.data[1][0] - self.data[0][0] * self.data[1][2]) * inv_det,
                (self.data[1][0] * self.data[2][1] - self.data[1][1] * self.data[2][0]) * inv_det,
                (self.data[0][1] * self.data[2][0] - self.data[0][0] * self.data[2][1]) * inv_det,
                (self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]) * inv_det,
            ))
        }
    }

    pub fn inverse(&self) -> Mat3 {
        self.try_inverse().expect("Matrix is not invertible")
    }

    pub fn transpose(&self) -> Mat3 {
        Mat3::from_cols(self.col(0), self.col(1), self.col(2))
    }
}

impl fmt::Display for Mat3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[[{}, {}, {}], [{}, {}, {}], [{}, {}, {}]]",
            self.data[0][0],
            self.data[0][1],
            self.data[0][2],
            self.data[1][0],
            self.data[1][1],
            self.data[1][2],
            self.data[2][0],
            self.data[2][1],
            self.data[2][2]
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
