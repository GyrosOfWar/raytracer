use std::ops::Mul;

use crate::vec::Vec3;

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
