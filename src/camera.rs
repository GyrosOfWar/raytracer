use std::time::Instant;

use num_traits::{One, Zero};

use crate::{
    helpers::random,
    ppm::Image,
    ray::Ray,
    trace::{Hittable, Range},
    vec3::{self, Point3, Vec3},
};

const MAX_DEPTH: usize = 10;

pub struct Camera {
    samples_per_pixel: usize,
    image_width: usize,
    image_height: usize,
    center: Point3<f32>,
    pixel_00_loc: Point3<f32>,
    pixel_delta_u: Vec3<f32>,
    pixel_delta_v: Vec3<f32>,
}

impl Camera {
    pub fn new(width: usize, height: usize, samples_per_pixel: usize) -> Self {
        assert!(
            samples_per_pixel >= 1,
            "must take at least one sample per pixel"
        );

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
            samples_per_pixel,
        }
    }

    fn ray_color(&self, ray: &Ray<f32>, depth: usize, world: &impl Hittable) -> Vec3<f32> {
        if depth <= 0 {
            return Vec3::zero();
        }

        let intersection = world.hit(ray, Range::new(0.0, f32::INFINITY));
        match intersection {
            Some(hit) => {
                let direction = vec3::random::gen_on_hemisphere(hit.normal);
                self.ray_color(&Ray::new(hit.point, direction), depth - 1, world) * 0.5
            }
            None => {
                let direction = ray.direction.unit();
                let t = 0.5 * (direction.y + 1.0);
                Vec3::one().lerp(Vec3::new(0.5, 0.7, 1.0), t)
            }
        }
    }

    // Construct a camera ray originating from the origin and directed at randomly sampled
    // point around the pixel location i, j.
    fn get_ray(&self, i: usize, j: usize) -> Ray<f32> {
        let offset = self.sample_square();
        let pixel_sample = self.pixel_00_loc
            + (self.pixel_delta_u * (i as f32 + offset.x))
            + (self.pixel_delta_v * (j as f32 + offset.y));

        let ray_origin = self.center;
        let ray_direction = pixel_sample - ray_origin;

        return Ray::new(ray_origin, ray_direction);
    }

    // Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square.
    fn sample_square(&self) -> Vec3<f32> {
        return Vec3::new(random() - 0.5, random() - 0.5, 0.0);
    }

    pub fn render(&self, world: &impl Hittable) -> Image {
        let start = Instant::now();
        let mut pixels = vec![];
        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f32;
        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let mut color = Vec3::zero();
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    color += self.ray_color(&ray, MAX_DEPTH, world);
                }
                color = color * pixel_samples_scale;

                pixels.push(color.into());
            }
        }
        let elapsed = start.elapsed();
        println!("rendering took {elapsed:?}");

        Image::new(pixels, self.image_width, self.image_height)
    }
}
