use std::ops::Mul;

use crate::vec::{vec3, Mat4, Point3, Vec3};

#[derive(Debug, Clone)]
pub struct Transform {
    matrix: Mat4,
    inverse: Mat4,
}

impl Transform {
    pub fn new(matrix: Mat4, inverse: Mat4) -> Self {
        Transform { inverse, matrix }
    }

    pub fn identity() -> Self {
        Transform::new(Mat4::IDENTITY, Mat4::IDENTITY)
    }

    pub fn from_matrix(matrix: Mat4) -> Self {
        Transform::new(matrix, matrix.inverse())
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        Transform::new(
            Mat4::from_translation(vec3(x, y, z)),
            Mat4::from_translation(vec3(-x, -y, -z)),
        )
    }

    pub fn rotate_x(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_x(angle), Mat4::from_rotation_x(-angle))
    }

    pub fn rotate_y(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_y(angle), Mat4::from_rotation_y(-angle))
    }

    pub fn rotate_z(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_z(angle), Mat4::from_rotation_z(-angle))
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Transform::new(
            Mat4::from_scale(vec3(x, y, z)),
            Mat4::from_scale(vec3(1.0 / x, 1.0 / y, 1.0 / z)),
        )
    }

    pub fn uniform_scale(s: f32) -> Self {
        Transform::scale(s, s, s)
    }

    pub fn look_at(pos: Point3, look: Point3, up: Vec3) -> Self {
        let mut world_from_camera = Mat4::IDENTITY;
        world_from_camera.set(0, 3, pos.x);
        world_from_camera.set(1, 3, pos.y);
        world_from_camera.set(2, 3, pos.z);
        world_from_camera.set(3, 3, 1.0);

        let dir = (look - pos).normalized();
        let right = up.normalized().cross(&dir).normalized();
        let new_up = dir.cross(&right).normalized();
        world_from_camera.set(0, 0, right.x);
        world_from_camera.set(1, 0, right.y);
        world_from_camera.set(2, 0, right.z);
        world_from_camera.set(3, 0, 0.);
        world_from_camera.set(0, 1, new_up.x);
        world_from_camera.set(1, 1, new_up.y);
        world_from_camera.set(2, 1, new_up.z);
        world_from_camera.set(3, 1, 0.);
        world_from_camera.set(0, 2, dir.x);
        world_from_camera.set(1, 2, dir.y);
        world_from_camera.set(2, 2, dir.z);
        world_from_camera.set(3, 2, 0.);

        let camera_from_world = world_from_camera.inverse();

        Transform::new(camera_from_world, world_from_camera)
    }

    pub fn orthographic(z_near: f32, z_far: f32) -> Self {
        Transform::scale(1.0, 1.0, 1.0 / (z_far - z_near)) * Transform::translate(0.0, 0.0, -z_near)
    }

    pub fn transform_point(&self, p: Point3) -> Point3 {
        let m = self.matrix;
        let xp = m.get(0, 0) * p.x + m.get(0, 1) * p.y + m.get(0, 2) * p.z + m.get(0, 3);
        let yp = m.get(1, 0) * p.x + m.get(1, 1) * p.y + m.get(1, 2) * p.z + m.get(1, 3);
        let zp = m.get(2, 0) * p.x + m.get(2, 1) * p.y + m.get(2, 2) * p.z + m.get(2, 3);
        let wp = m.get(3, 0) * p.x + m.get(3, 1) * p.y + m.get(3, 2) * p.z + m.get(3, 3);
        if wp == 1.0 {
            Point3::new(xp, yp, zp)
        } else {
            Point3::new(xp, yp, zp) / wp
        }
    }

    pub fn transform_point_inverse(&self, p: Point3) -> Point3 {
        let m = self.inverse;
        let xp = m.get(0, 0) * p.x + m.get(0, 1) * p.y + m.get(0, 2) * p.z + m.get(0, 3);
        let yp = m.get(1, 0) * p.x + m.get(1, 1) * p.y + m.get(1, 2) * p.z + m.get(1, 3);
        let zp = m.get(2, 0) * p.x + m.get(2, 1) * p.y + m.get(2, 2) * p.z + m.get(2, 3);
        let wp = m.get(3, 0) * p.x + m.get(3, 1) * p.y + m.get(3, 2) * p.z + m.get(3, 3);
        if wp == 1.0 {
            Point3::new(xp, yp, zp)
        } else {
            Point3::new(xp, yp, zp) / wp
        }
    }

    pub fn transform_vector(&self, v: Vec3) -> Vec3 {
        let m = self.matrix;
        Vec3::new(
            m.get(0, 0) * v.x + m.get(0, 1) * v.y + m.get(0, 2) * v.z,
            m.get(1, 0) * v.x + m.get(1, 1) * v.y + m.get(1, 2) * v.z,
            m.get(2, 0) * v.x + m.get(2, 1) * v.y + m.get(2, 2) * v.z,
        )
    }

    pub fn inverse(&self) -> Transform {
        Self {
            matrix: self.inverse,
            inverse: self.matrix,
        }
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform {
            matrix: self.matrix * rhs.matrix,
            inverse: rhs.inverse * self.inverse,
        }
    }
}