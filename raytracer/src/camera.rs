use crate::bounds::Bounds2f;
use crate::film::RgbFilm;
use crate::ray::{Ray, RayDifferential};
use crate::spectrum::SampledWavelengths;
use crate::transform::Transform;
use crate::vec::{Mat4, Point3, Vec2, Vec3};

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
        film: RgbFilm,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let ndc_from_screen =
            Transform::scale(
                1.0 / (screen_window.p_max().x - screen_window.p_min().x),
                1.0 / (screen_window.p_max().y - screen_window.p_min().y),
                1.0,
            ) * Transform::translate(-screen_window.p_min().x, -screen_window.p_max().y, 0.0);
        let raster_from_ndc = Transform::scale(
            film.full_resolution().x as f32,
            film.full_resolution().y as f32,
            1.0,
        );
        let raster_from_screen = raster_from_ndc * ndc_from_screen;
        let screen_from_raster = raster_from_screen.inverse();
        let camera_from_raster = screen_from_camera.inverse() * screen_from_raster.clone();

        PerspectiveCamera {
            camera_transform,
            screen_from_camera,
            camera_from_raster,
            raster_from_screen,
            screen_from_raster,
            lens_radius,
            focal_distance,
            z_near,
            z_far,
            film,
        }
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

#[derive(Debug)]
pub struct CameraTransform {
    world_from_render: Transform,
    render_from_camera: Transform,
}

impl CameraTransform {
    pub fn new(world_from_camera: Mat4) -> Self {
        let p_camera = world_from_camera.vec_mul(Vec3::ZERO);
        let world_from_render = Mat4::from_translation(p_camera);
        let render_from_world = world_from_render.inverse();
        let render_from_camera = render_from_world * world_from_camera;
        Self {
            world_from_render: Transform::new(world_from_render, render_from_world),
            render_from_camera: Transform::new(render_from_camera, render_from_camera.inverse()),
        }
    }

    pub fn render_from_camera(&self, p: Point3) -> Point3 {
        self.render_from_camera.transform_point(p)
    }

    pub fn camera_from_render(&self, p: Point3) -> Point3 {
        self.render_from_camera.transform_point_inverse(p)
    }

    pub fn render_from_world(&self, p: Point3) -> Point3 {
        self.world_from_render.transform_point_inverse(p)
    }

    pub fn camera_from_world(&self) -> Transform {
        (self.world_from_render.clone() * self.render_from_camera.clone()).inverse()
    }
}
