use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use num_traits::{One, Zero};

use crate::{
    helpers::random,
    material::Scatterable,
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
    defocus_disk_u: Vec3<f32>,
    defocus_disk_v: Vec3<f32>,
    defocus_angle: f32,
}

impl Camera {
    pub fn new(
        width: usize,
        height: usize,
        samples_per_pixel: usize,
        look_from: Point3<f32>,
        look_at: Point3<f32>,
        defocus_angle: f32,
        focus_dist: f32,
    ) -> Self {
        assert!(
            samples_per_pixel >= 1,
            "must take at least one sample per pixel"
        );

        let v_up = Vec3::new(0.0, 1.0, 0.0);
        let v_fov = 45.0f32;

        let theta = v_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;
        let viewport_width = viewport_height * (width as f32 / height as f32);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (look_from - look_at).unit();
        let u = v_up.cross(w).unit();
        let v = w.cross(u);

        let camera_center = look_from;
        let viewport_u = u * viewport_width;
        let viewport_v = -v * viewport_height;

        let pixel_delta_u = viewport_u / width as f32;
        let pixel_delta_v = viewport_v / height as f32;

        let viewport_upper_left =
            camera_center - (w * focus_dist) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

        let defocus_radius = focus_dist * (defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Camera {
            image_height: height,
            image_width: width,
            center: camera_center,
            pixel_00_loc,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel,
            defocus_disk_u,
            defocus_disk_v,
            defocus_angle,
        }
    }

    fn ray_color(&self, ray: &Ray<f32>, depth: usize, world: &impl Hittable) -> Vec3<f32> {
        if depth <= 0 {
            return Vec3::zero();
        }

        let intersection = world.hit(ray, Range::new(0.001, f32::INFINITY));
        match intersection {
            Some(hit) => {
                let mut scattered = Ray::new(Vec3::zero(), Vec3::zero());
                let mut attenuation = Vec3::zero();
                if hit
                    .material
                    .scatter(ray, &hit, &mut attenuation, &mut scattered)
                {
                    attenuation * self.ray_color(&scattered, depth - 1, world)
                } else {
                    Vec3::zero()
                }
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

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;

        return Ray::new(ray_origin, ray_direction);
    }

    fn defocus_disk_sample(&self) -> Vec3<f32> {
        // Returns a random point in the camera defocus disk.
        // let p = random_in_unit_disk();
        let p = vec3::random::gen_unit_disk();
        return self.center + (self.defocus_disk_u * p.x) + (self.defocus_disk_v * p.y);
    }

    // Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square.
    fn sample_square(&self) -> Vec3<f32> {
        return Vec3::new(random() - 0.5, random() - 0.5, 0.0);
    }

    pub fn render(&self, world: &impl Hittable) -> Image {
        let start = Instant::now();
        let mut pixels = vec![];
        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f32;
        let progress = ProgressBar::new(
            (self.image_height * self.image_width * self.samples_per_pixel) as u64,
        );
        progress.set_style(
            ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
                .unwrap(),
        );
        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let mut color = Vec3::zero();
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    color += self.ray_color(&ray, MAX_DEPTH, world);
                    progress.inc(1);
                }
                color = color * pixel_samples_scale;
                pixels.push(color.into());
            }
        }
        progress.finish_and_clear();
        let elapsed = start.elapsed();
        println!("rendering took {elapsed:?}");

        Image::new(pixels, self.image_width, self.image_height)
    }
}
