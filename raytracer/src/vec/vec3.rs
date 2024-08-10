use std::fmt;
use std::ops::Neg;

use super::VectorLike;
use crate::impl_binary_op;
use crate::vec::Axis;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
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
        f32::mul_add(self.x, rhs.x, f32::mul_add(self.y, rhs.y, self.z * rhs.z))
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

impl From<Point3> for Vec3 {
    fn from(value: Point3) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

impl From<Vec3> for Point3 {
    fn from(value: Vec3) -> Self {
        Point3::new(value.x, value.y, value.z)
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub const ZERO: Point3 = Point3 {
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

    pub fn get(&self, i: usize) -> f32 {
        assert!(i < 3, "index out of bounds");
        match i {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => unreachable!(),
        }
    }
}

impl VectorLike<3, f32> for Point3 {
    fn component(&self, index: usize) -> f32 {
        self.get(index)
    }

    fn data(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    fn from_data(data: [f32; 3]) -> Self {
        Point3::new(data[0], data[1], data[2])
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

impl_binary_op!(Add : add => (lhs: Point3, rhs: Point3) -> Vec3 {
    Vec3::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
        lhs.z + rhs.z,
    )
});

impl_binary_op!(Sub : sub => (lhs: Point3, rhs: Point3) -> Vec3 {
    Vec3::new(
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

impl VectorLike<3, f32> for Vec3 {
    fn component(&self, index: usize) -> f32 {
        self.get(index)
    }

    fn data(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    fn from_data(data: [f32; 3]) -> Self {
        Vec3::new(data[0], data[1], data[2])
    }
}

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - n * Vec3::dot(&v, n) * 2.0
}

pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parallel = n * -(1.0 - r_out_perp.length_squared()).abs().sqrt();
    r_out_perp + r_out_parallel
}

#[cfg(test)]
mod tests {
    use crate::vec::Vec3;

    #[test]
    fn test_vec3_dot() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a.dot(b), 4.0 + 10.0 + 18.0);
    }
}
