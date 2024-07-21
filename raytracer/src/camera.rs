use std::ops::Mul;

use glam::{Mat3A, Mat4, Vec2, Vec3A};

use crate::{ray::Ray, spectrum::SampledWavelengths};

pub struct CameraSample {
    pub p_film: Vec2,
    pub p_lens: Vec2,
    pub filter_weight: f32,
}

pub struct PerspectiveCamera {
    transform: CameraTransform,
}

impl PerspectiveCamera {
    pub fn generate_ray(sample: CameraSample, lambda: &mut SampledWavelengths) -> Option<Ray> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Transform {
    matrix: Mat4,
    inverse: Mat4,
}

impl Transform {
    pub fn new(matrix: Mat4) -> Self {
        Transform {
            inverse: matrix.inverse(),
            matrix,
        }
    }

    pub fn transform(&self, point: Vec3A) -> Vec3A {
        self.matrix.transform_point3a(point)
    }

    pub fn inverse_transform(&self, point: Vec3A) -> Vec3A {
        self.inverse.transform_point3a(point)
    }

    pub fn inverse(self) -> Transform {
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
            inverse: self.inverse * rhs.inverse,
        }
    }
}

pub struct CameraTransform {
    world_from_render: Transform,
    render_from_camera: Transform,
}

impl CameraTransform {
    pub fn new(world_from_camera: Mat4) -> Self {
        let p_camera = world_from_camera.transform_vector3a(Vec3A::ZERO);
        let world_from_render = Mat4::from_translation(p_camera.into());
        let render_from_world = world_from_render.inverse();
        let render_from_camera = render_from_world * world_from_camera;
        Self {
            world_from_render: Transform::new(world_from_render),
            render_from_camera: Transform::new(render_from_camera),
        }
    }

    pub fn render_from_camera(&self, p: Vec3A) -> Vec3A {
        self.render_from_camera.transform(p)
    }

    pub fn camera_from_render(&self, p: Vec3A) -> Vec3A {
        self.render_from_camera.inverse_transform(p)
    }

    pub fn render_from_world(&self, p: Vec3A) -> Vec3A {
        self.world_from_render.inverse_transform(p)
    }

    pub fn camera_from_world(&self) -> Transform {
        (self.world_from_render.clone() * self.render_from_camera.clone()).inverse()
    }
}
