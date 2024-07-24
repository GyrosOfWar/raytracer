use std::ops::Mul;

use glam::{vec3, I64Vec2, Mat4, Vec2, Vec3A};

use crate::film::RgbFilm;
use crate::ray::{Ray, RayDifferential};
use crate::spectrum::SampledWavelengths;

pub struct Bounds2f {
    p_min: Vec2,
    p_max: Vec2,
}

impl Bounds2f {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Bounds2f {
            p_min: a.min(b),
            p_max: a.max(b),
        }
    }

    pub fn p_min(&self) -> Vec2 {
        self.p_min
    }

    pub fn p_max(&self) -> Vec2 {
        self.p_max
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bounds2i {
    p_min: I64Vec2,
    p_max: I64Vec2,
}

impl Bounds2i {
    pub fn new(a: I64Vec2, b: I64Vec2) -> Self {
        Bounds2i {
            p_min: a.min(b),
            p_max: a.max(b),
        }
    }

    pub fn p_min(&self) -> I64Vec2 {
        self.p_min
    }

    pub fn p_max(&self) -> I64Vec2 {
        self.p_max
    }

    pub fn area(&self) -> i64 {
        let d = self.p_max - self.p_min;
        d.x * d.y
    }

    pub fn x_extent(&self) -> i64 {
        self.p_max.x - self.p_min.x
    }

    pub fn y_extent(&self) -> i64 {
        self.p_max.y - self.p_min.y
    }
}

pub struct CameraSample {
    pub p_film: Vec2,
    pub p_lens: Vec2,
    pub filter_weight: f32,
}

#[derive(Debug)]
pub struct PerspectiveCamera {
    camera_transform: CameraTransform,
    screen_from_camera: Transform,
    camera_from_raster: Transform,
    raster_from_screen: Transform,
    screen_from_raster: Transform,
    lens_radius: f32,
    focal_distance: f32,
    z_near: f32,
    z_far: f32,
    // does it need to own the film?
    film: RgbFilm,
}

impl PerspectiveCamera {
    pub fn new(
        camera_transform: CameraTransform,
        screen_from_camera: Transform,
        screen_window: Bounds2f,
        lens_radius: f32,
        focal_distance: f32,
    ) -> Self {
        let ndc_from_screen =
            Transform::scale(
                1.0 / (screen_window.p_max().x - screen_window.p_min().x),
                1.0 / (screen_window.p_max().y - screen_window.p_min().y),
                1.0,
            ) * Transform::translate(-screen_window.p_min().x, -screen_window.p_max().y, 0.0);
        // let raster_from_ndc = Transform::scale();
        todo!()
    }

    pub fn generate_ray(
        &self,
        sample: CameraSample,
        lambda: &mut SampledWavelengths,
    ) -> Option<Ray> {
        todo!()
    }

    pub fn generate_ray_differential(
        &self,
        sample: CameraSample,
        lambda: &mut SampledWavelengths,
    ) -> Option<RayDifferential> {
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

    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        Transform::new(Mat4::from_translation(vec3(x, y, z)))
    }

    pub fn rotate_x(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_x(angle))
    }

    pub fn rotate_y(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_y(angle))
    }

    pub fn rotate_z(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_z(angle))
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Transform::new(Mat4::from_scale(vec3(x, y, z)))
    }

    pub fn uniform_scale(s: f32) -> Self {
        Transform::scale(s, s, s)
    }

    pub fn apply(&self, point: Vec3A) -> Vec3A {
        self.matrix.transform_point3a(point)
    }

    pub fn apply_inverse(&self, point: Vec3A) -> Vec3A {
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

#[derive(Debug)]
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
        self.render_from_camera.apply(p)
    }

    pub fn camera_from_render(&self, p: Vec3A) -> Vec3A {
        self.render_from_camera.apply_inverse(p)
    }

    pub fn render_from_world(&self, p: Vec3A) -> Vec3A {
        self.world_from_render.apply_inverse(p)
    }

    pub fn camera_from_world(&self) -> Transform {
        (self.world_from_render.clone() * self.render_from_camera.clone()).inverse()
    }
}
