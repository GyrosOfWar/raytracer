use std::fmt;
use std::ops::{Mul, Neg};

use crate::vec::Axis;

macro_rules! impl_binary_op {
    ($op:tt : $method:ident => (
           $lhs_i:ident : $lhs_t:path,
           $rhs_i:ident : $rhs_t:path
        ) -> $return_t:path $body:block
    ) => {
        impl std::ops::$op<$rhs_t> for $lhs_t {
            type Output = $return_t;

            fn $method(self, $rhs_i: $rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl std::ops::$op<&$rhs_t> for $lhs_t {
            type Output = $return_t;

            fn $method(self, $rhs_i: &$rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl std::ops::$op<$rhs_t> for &$lhs_t {
            type Output = $return_t;

            fn $method(self, $rhs_i: $rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
        impl std::ops::$op<&$rhs_t> for &$lhs_t {
            type Output = $return_t;

            fn $method(self, $rhs_i: &$rhs_t) -> $return_t {
                let $lhs_i = self;
                $body
            }
        }
    };
}

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn min(&self, b: Vec2) -> Vec2 {
        todo!()
    }

    pub fn max(&self, b: Vec2) -> Vec2 {
        todo!()
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl_binary_op!(Mul : mul => (lhs: Vec2, rhs: Vec2) -> Vec2 {
    Vec2::new(
        lhs.x * rhs.x,
        lhs.y * rhs.y,
    )
});

impl_binary_op!(Mul : mul => (lhs: Vec2, rhs: f32) -> Vec2 {
    Vec2::new(
        lhs.x * rhs,
        lhs.y * rhs,
    )
});

#[derive(Debug, Copy, Clone)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Point2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ONE: Vec3 = Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        assert!(!x.is_nan(), "x must be finite");
        assert!(!y.is_nan(), "y must be finite");
        assert!(!z.is_nan(), "z must be finite");

        Self { x, y, z }
    }

    pub fn at(&self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    pub fn all(v: f32) -> Vec3 {
        Vec3::new(v, v, v)
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn dot(&self, rhs: Vec3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub fn normalized(self) -> Vec3 {
        self * self.length().recip()
    }

    pub fn get(&self, i: usize) -> f32 {
        assert!(i < 3, "index out of bounds");
        match i {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => unreachable!(),
        }
    }

    pub fn abs(&self) -> Vec3 {
        Vec3::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn abs_diff_eq(&self, rhs: Vec3, eps: f32) -> bool {
        let diff = (self - rhs).abs();
        diff.x <= eps && diff.y <= eps && diff.z <= eps
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        assert!(!x.is_nan(), "x must be finite");
        assert!(!y.is_nan(), "y must be finite");
        assert!(!z.is_nan(), "z must be finite");

        Self { x, y, z }
    }

    pub fn at(&self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }
}

impl_binary_op!(Add : add => (lhs: Vec3, rhs: Vec3) -> Vec3 {
    Vec3::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
        lhs.z + rhs.z,
    )
});

impl_binary_op!(Sub : sub => (lhs: Vec3, rhs: Vec3) -> Vec3 {
    Vec3::new(
        lhs.x - rhs.x,
        lhs.y - rhs.y,
        lhs.z - rhs.z,
    )
});

impl_binary_op!(Mul : mul => (lhs: Vec3, rhs: f32) -> Vec3 {
    Vec3::new(
        lhs.x * rhs,
        lhs.y * rhs,
        lhs.z * rhs,
    )
});

impl_binary_op!(Mul : mul => (lhs: f32, rhs: Vec3) -> Vec3 {
    Vec3::new(
        rhs.x * lhs,
        rhs.y * lhs,
        rhs.z * lhs,
    )
});

impl_binary_op!(Div : div => (lhs: Vec3, rhs: f32) -> Vec3 {
    Vec3::new(
        lhs.x / rhs,
        lhs.y / rhs,
        lhs.z / rhs,
    )
});

impl_binary_op!(Add : add => (lhs: Point3, rhs: Point3) -> Point3 {
    Point3::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
        lhs.z + rhs.z,
    )
});

impl_binary_op!(Sub : sub => (lhs: Point3, rhs: Point3) -> Point3 {
    Point3::new(
        lhs.x - rhs.x,
        lhs.y - rhs.y,
        lhs.z - rhs.z,
    )
});

impl_binary_op!(Mul : mul => (lhs: Point3, rhs: f32) -> Point3 {
    Point3::new(
        lhs.x * rhs,
        lhs.y * rhs,
        lhs.z * rhs,
    )
});

impl_binary_op!(Mul : mul => (lhs: f32, rhs: Point3) -> Point3 {
    Point3::new(
        rhs.x * lhs,
        rhs.y * lhs,
        rhs.z * lhs,
    )
});

impl_binary_op!(Div : div => (lhs: Point3, rhs: f32) -> Point3 {
    Point3::new(
        lhs.x / rhs,
        lhs.y / rhs,
        lhs.z / rhs,
    )
});

impl_binary_op!(Add : add => (lhs: Point3, rhs: Vec3) -> Point3 {
    Point3::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
        lhs.z + rhs.z,
    )
});

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

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

#[derive(Debug, Copy, Clone)]
pub struct Mat4 {
    data: [f32; 16],
}

impl Mat4 {
    pub fn inverse(&self) -> Mat4 {
        todo!()
    }

    pub fn from_translation(z: Vec3) -> Mat4 {
        todo!()
    }

    pub fn from_rotation_x(angle: f32) -> Mat4 {
        todo!()
    }

    pub fn from_rotation_y(angle: f32) -> Mat4 {
        todo!()
    }

    pub fn from_rotation_z(angle: f32) -> Mat4 {
        todo!()
    }

    pub fn from_scale(z: Vec3) -> Mat4 {
        todo!()
    }

    pub fn transform_point3(&self, point: Vec3) -> Vec3 {
        todo!()
    }

    pub fn transform_vector3(&self, vector: Vec3) -> Vec3 {
        todo!()
    }

    pub fn mat_mul(&self, rhs: Mat4) -> Mat4 {
        todo!()
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mat_mul(rhs)
    }
}

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

pub fn point2(x: f32, y: f32) -> Point2 {
    Point2::new(x, y)
}

pub fn point3(x: f32, y: f32, z: f32) -> Point3 {
    Point3::new(x, y, z)
}

pub fn ivec2(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

pub fn uvec2(x: u32, y: u32) -> UVec2 {
    UVec2::new(x, y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub(crate) fn min(&self, b: IVec2) -> IVec2 {
        todo!()
    }

    pub(crate) fn max(&self, b: IVec2) -> IVec2 {
        todo!()
    }
}

impl_binary_op!(Add : add => (lhs: IVec2, rhs: IVec2) -> IVec2 {
    IVec2::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
    )
});

impl_binary_op!(Sub : sub => (lhs: IVec2, rhs: IVec2) -> IVec2 {
    IVec2::new(
        lhs.x - rhs.x,
        lhs.y - rhs.y,
    )
});

#[derive(Debug, Copy, Clone)]
pub struct UVec2 {
    pub x: u32,
    pub y: u32,
}

impl UVec2 {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl_binary_op!(Add : add => (lhs: UVec2, rhs: UVec2) -> UVec2 {
    UVec2::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
    )
});

impl_binary_op!(Sub : sub => (lhs: UVec2, rhs: UVec2) -> UVec2 {
    UVec2::new(
        lhs.x - rhs.x,
        lhs.y - rhs.y,
    )
});
