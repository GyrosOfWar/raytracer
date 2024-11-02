use std::fmt;
use std::ops::{Index, Mul};

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

    #[allow(clippy::too_many_arguments)]
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

    pub fn from_cols(c0: [f32; 4], c1: [f32; 4], c2: [f32; 4], c3: [f32; 4]) -> Self {
        Self {
            data: [
                [c0[0], c1[0], c2[0], c3[0]],
                [c0[1], c1[1], c2[1], c3[1]],
                [c0[2], c1[2], c2[2], c3[2]],
                [c0[3], c1[3], c2[3], c3[3]],
            ],
        }
    }

    pub fn col(&self, j: usize) -> [f32; 4] {
        [
            self.data[0][j],
            self.data[1][j],
            self.data[2][j],
            self.data[3][j],
        ]
    }

    pub fn try_inverse(&self) -> Option<Mat4> {
        let a = self.data[0][0];
        let b = self.data[0][1];
        let c = self.data[0][2];
        let d = self.data[0][3];
        let e = self.data[1][0];
        let f = self.data[1][1];
        let g = self.data[1][2];
        let h = self.data[1][3];
        let i = self.data[2][0];
        let j = self.data[2][1];
        let k = self.data[2][2];
        let l = self.data[2][3];
        let m = self.data[3][0];
        let n = self.data[3][1];
        let o = self.data[3][2];
        let p = self.data[3][3];

        let q = f * (k * p - l * o) - g * (j * p - l * n) + h * (j * o - k * n);
        let r = e * (k * p - l * o) - g * (i * p - l * m) + h * (i * o - k * m);
        let s = e * (j * p - l * n) - f * (i * p - l * m) + h * (i * n - j * m);
        let t = e * (j * o - k * n) - f * (i * o - k * m) + g * (i * n - j * m);

        let u = b * (k * p - l * o) - c * (j * p - l * n) + d * (j * o - k * n);
        let v = a * (k * p - l * o) - c * (i * p - l * m) + d * (i * o - k * m);
        let w = a * (j * p - l * n) - b * (i * p - l * m) + d * (i * n - j * m);
        let x = a * (j * o - k * n) - b * (i * o - k * m) + c * (i * n - j * m);

        let y = b * (g * p - h * o) - c * (f * p - h * n) + d * (f * o - g * n);
        let z = a * (g * p - h * o) - c * (e * p - h * m) + d * (e * o - g * m);
        let aa = a * (f * p - h * n) - b * (e * p - h * m) + d * (e * n - f * m);
        let ab = a * (f * o - g * n) - b * (e * o - g * m) + c * (e * n - f * m);

        let ac = b * (g * l - h * k) - c * (f * l - h * j) + d * (f * k - g * j);
        let ad = a * (g * l - h * k) - c * (e * l - h * i) + d * (e * k - g * i);
        let ae = a * (f * l - h * j) - b * (e * l - h * i) + d * (e * j - f * i);
        let af = a * (f * k - g * j) - b * (e * k - g * i) + c * (e * j - f * i);

        let det = a * q - b * r + c * s - d * t;

        if det == 0.0 {
            None
        } else {
            let inv_det = 1.0 / det;

            let data: [[f32; 4]; 4] = [
                [q * inv_det, -u * inv_det, y * inv_det, -ac * inv_det],
                [-r * inv_det, v * inv_det, -z * inv_det, ad * inv_det],
                [s * inv_det, -w * inv_det, aa * inv_det, -ae * inv_det],
                [-t * inv_det, x * inv_det, -ab * inv_det, af * inv_det],
            ];

            Some(Mat4 { data })
        }
    }

    pub fn inverse(&self) -> Mat4 {
        self.try_inverse().expect("Matrix is not invertible")
    }

    pub fn transpose(&self) -> Mat4 {
        Mat4::from_cols(self.col(0), self.col(1), self.col(2), self.col(3))
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

impl fmt::Display for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[[{}, {}, {}, {}]\n [{}, {}, {}, {}]\n [{}, {}, {}, {}]\n [{}, {}, {}, {}]]",
            self.data[0][0],
            self.data[0][1],
            self.data[0][2],
            self.data[0][3],
            self.data[1][0],
            self.data[1][1],
            self.data[1][2],
            self.data[1][3],
            self.data[2][0],
            self.data[2][1],
            self.data[2][2],
            self.data[2][3],
            self.data[3][0],
            self.data[3][1],
            self.data[3][2],
            self.data[3][3]
        )
    }
}

impl Index<usize> for Mat4 {
    type Output = [f32; 4];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use super::Mat4;
    use crate::assert_approx_eq;
    use crate::random::random;

    fn random_mat4() -> Mat4 {
        Mat4::new(
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
            random(),
        )
    }

    #[test]
    fn test_invert() {
        let matrix = random_mat4();
        let inverse = matrix.inverse();
        let identity = matrix * inverse;

        let eps = 1e-5;
        assert_approx_eq!(identity.get(0, 0), 1.0, eps);
        assert_approx_eq!(identity.get(0, 1), 0.0, eps);
        assert_approx_eq!(identity.get(0, 2), 0.0, eps);
        assert_approx_eq!(identity.get(0, 3), 0.0, eps);

        assert_approx_eq!(identity.get(1, 0), 0.0, eps);
        assert_approx_eq!(identity.get(1, 1), 1.0, eps);
        assert_approx_eq!(identity.get(1, 2), 0.0, eps);
        assert_approx_eq!(identity.get(1, 3), 0.0, eps);

        assert_approx_eq!(identity.get(2, 0), 0.0, eps);
        assert_approx_eq!(identity.get(2, 1), 0.0, eps);
        assert_approx_eq!(identity.get(2, 2), 1.0, eps);
        assert_approx_eq!(identity.get(2, 3), 0.0, eps);

        assert_approx_eq!(identity.get(3, 0), 0.0, eps);
        assert_approx_eq!(identity.get(3, 1), 0.0, eps);
        assert_approx_eq!(identity.get(3, 2), 0.0, eps);
        assert_approx_eq!(identity.get(3, 3), 1.0, eps);
    }

    #[test]
    #[should_panic]
    fn test_invert_panic() {
        let matrix = Mat4::ZERO;
        matrix.inverse();
    }
}
