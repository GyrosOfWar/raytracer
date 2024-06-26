use std::time::Instant;

use num_traits::{One, Zero};

use crate::{
    ppm::{Color, Image},
    ray::Ray,
    trace::{Hittable, Range},
    vec3::{Point3, Vec3},
};

pub struct Camera {
    image_width: usize,
    image_height: usize,
    center: Point3<f32>,
    pixel_00_loc: Point3<f32>,
    pixel_delta_u: Vec3<f32>,
    pixel_delta_v: Vec3<f32>,
}

impl Camera {
    pub fn new(width: usize, height: usize) -> Self {
        let focal_length = 1.0f32;
        let viewport_height = 2.0f32;
        let viewport_width = viewport_height * (width as f32 / height as f32);
        let camera_center = Point3::zero();
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);
        let pixel_delta_u = viewport_u / width as f32;
        let pixel_delta_v = viewport_v / height as f32;

        let viewport_upper_left =
            camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

        Camera {
            image_height: height,
            image_width: width,
            center: camera_center,
            pixel_00_loc,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    fn ray_color(&self, ray: &Ray<f32>, world: &impl Hittable) -> Color {
        let intersection = world.hit(ray, Range::new(0.0, f32::INFINITY));
        match intersection {
            Some(hit) => {
                let n = (hit.point - Vec3::new(0.0, 0.0, -1.0)).unit();
                ((n + 1.0) * 0.5).into()
            }
            None => {
                let direction = ray.direction.unit();
                let t = 0.5 * (direction.y + 1.0);
                Vec3::one().lerp(Vec3::new(0.5, 0.7, 1.0), t).into()
            }
        }
    }

    pub fn render(&self, world: &impl Hittable) -> Image {
        let start = Instant::now();
        let mut pixels = vec![];
        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let pixel_center = self.pixel_00_loc
                    + (self.pixel_delta_u * i as f32)
                    + (self.pixel_delta_v * j as f32);
                let ray_direction = pixel_center - self.center;
                let ray = Ray::new(self.center, ray_direction);

                let color = self.ray_color(&ray, world);
                pixels.push(color);
            }
        }
        let elapsed = start.elapsed();
        println!("rendering took {elapsed:?}");

        Image::new(pixels, self.image_width, self.image_height)
    }
}
