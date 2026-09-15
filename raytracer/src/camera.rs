use num_traits::Zero;

use crate::bounds::Bounds2f;
use crate::film::{Film, RgbFilm};
use crate::filter::Filter;
use crate::ray::{Ray, RayDifferential, RayLike};
use crate::sample::sample_uniform_disk_concentric;
use crate::spectrum::SampledWavelengths;
use crate::transform::Transform;
use crate::vec::{Mat4, Point2, Point3, Vec2, Vec3};

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
    film: RgbFilm,

    dx_camera: Vec3,
    dy_camera: Vec3,
    cos_total_width: f32,
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
        let dx_camera = camera_from_raster.transform_point(Point3::new(1.0, 0.0, 0.0))
            - camera_from_raster.transform_point(Point3::new(0.0, 0.0, 0.0));
        let dy_camera = camera_from_raster.transform_point(Point3::new(0.0, 1.0, 0.0))
            - camera_from_raster.transform_point(Point3::new(0.0, 0.0, 0.0));
        let radius = Point2::from(film.filter().radius());
        let p_corner = Point3::new(-radius.x, -radius.y, 0.0);
        let w_corner_camera = Vec3::from(camera_from_raster.transform_point(p_corner)).normalized();
        let cos_total_width = w_corner_camera.z;

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
            dx_camera,
            dy_camera,
            cos_total_width,
        }
    }

    pub fn generate_ray(
        &self,
        sample: CameraSample,
        lambda: &mut SampledWavelengths,
    ) -> Option<Ray> {
        let p_film = Point3::new(sample.p_film.x, sample.p_film.y, 0.0);
        let p_camera = self.camera_from_raster.transform_point(p_film);
        let mut ray = Ray::new(Point3::zero(), Vec3::from(p_camera).normalized());
        if self.lens_radius > 0.0 {
            let p_lens = self.lens_radius * sample_uniform_disk_concentric(sample.p_lens);
            let ft = self.focal_distance / ray.direction.z;
            let p_focus = ray.evaluate(ft);

            ray.origin = Point3::new(p_lens.x, p_lens.y, 0.0);
            ray.direction = (p_focus - ray.origin).normalized();
        }

        Some(ray)
    }

    pub fn generate_ray_differential(
        &self,
        sample: CameraSample,
        lambda: &mut SampledWavelengths,
    ) -> Option<RayDifferential> {
        // Compute raster and camera sample positions
        let p_film = Point3::new(sample.p_film.x, sample.p_film.y, 0.0);
        let p_camera = self.camera_from_raster.transform_point(p_film);
        let dir = Vec3::from(p_camera).normalized();
        let mut ray = RayDifferential::new(Point3::zero(), dir);
        if self.lens_radius > 0.0 {
            let p_lens = self.lens_radius * sample_uniform_disk_concentric(sample.p_lens);
            let ft = self.focal_distance / ray.direction.z;
            let p_focus = ray.evaluate(ft);

            ray.origin = Point3::new(p_lens.x, p_lens.y, 0.0);
            ray.direction = (p_focus - ray.origin).normalized();
        }

        if self.lens_radius > 0.0 {
            let p_lens = self.lens_radius * sample_uniform_disk_concentric(sample.p_lens);

            let dx = Vec3::from(p_camera + self.dx_camera).normalized();
            let ft = self.focal_distance / dx.z;
            let p_focus = Point3::zero() + (dx * ft);
            ray.differential.rx_origin = Point3::new(p_lens.x, p_lens.y, 0.0);
            ray.differential.rx_direction = (p_focus - ray.differential.rx_origin).normalized();

            let dy = Vec3::from(p_camera + self.dy_camera).normalized();
            let ft = self.focal_distance / dy.z;
            let p_focus = Point3::zero() + (dy * ft);
            ray.differential.ry_origin = Point3::new(p_lens.x, p_lens.y, 0.0);
            ray.differential.ry_direction = (p_focus - ray.differential.ry_origin).normalized();
        } else {
            ray.differential.rx_origin = ray.origin;
            ray.differential.ry_origin = ray.origin;
            ray.differential.rx_direction = (Vec3::from(p_camera) + self.dx_camera).normalized();
            ray.differential.ry_direction = (Vec3::from(p_camera) + self.dy_camera).normalized();
        }

        Some(ray)
    }
}

#[derive(Debug)]
pub struct CameraTransform {
    world_from_render: Transform,
    render_from_camera: Transform,
}

impl CameraTransform {
    pub fn new(world_from_camera: Mat4) -> Self {
        let p_camera = world_from_camera.vec_mul(Vec3::zero());
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
