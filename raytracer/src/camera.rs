use glam::{vec3a, Mat4, Vec2, Vec3A};

use crate::{
    ray::Ray,
    spectrum::{SampledSpectrum, SampledWavelengths},
};

pub struct CameraSample {
    pub p_film: Vec2,
    pub p_lens: Vec2,
    pub filter_weight: f32,
}

pub struct PerspectiveCamera {}

impl PerspectiveCamera {
    pub fn generate_ray(sample: CameraSample, lambda: &mut SampledWavelengths) -> Option<Ray> {
        todo!()
    }
}

pub struct CameraTransform {
    world_from_render: Mat4,
    render_from_camera: Mat4,
}

impl CameraTransform {
    pub fn new(world_from_camera: Mat4) -> Self {
        let p_camera = world_from_camera.transform_vector3a(Vec3A::ZERO);
        let world_from_render = Mat4::from_translation(p_camera.into());
        Self {
            world_from_render,
            render_from_camera,
        }
    }
}
