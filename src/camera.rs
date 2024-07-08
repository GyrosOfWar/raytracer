use std::time::Instant;

use crate::material::{ScatterResult, Scatterable};
use clap::ValueEnum;
use image::{DynamicImage, Rgb32FImage, RgbImage};
use indicatif::ParallelProgressIterator;
use num_traits::Zero;
use rayon::prelude::*;
use tracing::info;

use crate::{
    object::Hittable,
    random::random,
    range::Range,
    ray::Ray,
    vec3::{self, Point3, Vec3},
};

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum RenderMode {
    Sequential,
    Parallel,
    ParallelChunks,
}

fn linear_to_gamma(linear_component: f32) -> f32 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

#[derive(Debug)]
pub struct CameraParams {
    pub image_size: (usize, usize),
    pub samples_per_pixel: usize,
    pub look_at: Point3<f32>,
    pub look_from: Point3<f32>,
    pub defocus_angle: f32,
    pub focus_dist: f32,
    pub vertical_fov: f32,
    pub max_depth: usize,
    pub background_color: Point3<f32>,
}

impl Default for CameraParams {
    fn default() -> Self {
        Self {
            image_size: (1280, 720),
            samples_per_pixel: 100,
            look_at: Point3::zero(),
            look_from: Point3::new(0.0, 0.0, -1.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
            vertical_fov: 90.0,
            max_depth: 50,
            background_color: Point3::new(0.5, 0.5, 0.5),
        }
    }
}

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
    max_depth: usize,
    background_color: Point3<f32>,
}

impl Camera {
    pub fn new(
        CameraParams {
            image_size,
            samples_per_pixel,
            look_at,
            look_from,
            defocus_angle,
            focus_dist,
            vertical_fov,
            max_depth,
            background_color,
        }: CameraParams,
    ) -> Self {
        assert!(
            samples_per_pixel >= 1,
            "must take at least one sample per pixel"
        );

        let v_up = Vec3::new(0.0, 1.0, 0.0);

        let theta = vertical_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;
        let (width, height) = image_size;
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
            max_depth,
            background_color,
        }
    }

    fn ray_color(&self, ray: &Ray<f32>, depth: usize, world: &impl Hittable) -> Vec3<f32> {
        if depth == 0 {
            return Vec3::zero();
        }

        let intersection = world.hit(ray, Range::new(0.001, f32::INFINITY));
        match intersection {
            Some(hit) => {
                let emitted_color = hit.material.emit(hit.tex_coords, hit.point);

                match hit.material.scatter(ray, &hit) {
                    Some(ScatterResult {
                        attenuation,
                        scattered,
                    }) => {
                        let scattered_color =
                            attenuation * self.ray_color(&scattered, depth - 1, world);
                        emitted_color + scattered_color
                    }
                    None => emitted_color,
                }
            }
            None => self.background_color,
        }
    }

    /// Construct a camera ray originating from the origin and directed at randomly sampled
    /// point around the pixel location `(i, j)`.
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

        Ray::new(ray_origin, ray_direction)
    }

    /// Returns a random point in the camera defocus disk.
    fn defocus_disk_sample(&self) -> Vec3<f32> {
        let p = vec3::random::gen_unit_disk();
        self.center + (self.defocus_disk_u * p.x) + (self.defocus_disk_v * p.y)
    }

    // Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square.
    fn sample_square(&self) -> Vec3<f32> {
        Vec3::new(random() - 0.5, random() - 0.5, 0.0)
    }

    fn render_parallel(&self, pixel_samples_scale: f32, world: &impl Hittable) -> Vec<f32> {
        let threads = rayon::current_num_threads();
        let pixel_count = self.image_height * self.image_width;
        info!("rendering in parallel with {threads} threads");
        (0..pixel_count)
            .into_par_iter()
            .progress_count(pixel_count as u64)
            .flat_map(|index| {
                let i = index % self.image_width;
                let j = index / self.image_width;
                let mut color = Vec3::zero();
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    color += self.ray_color(&ray, self.max_depth, world);
                }
                let result = color * pixel_samples_scale;
                [result.x, result.y, result.z]
            })
            .map(linear_to_gamma)
            .collect()
    }

    fn render_parallel_chunks(&self, pixel_samples_scale: f32, world: &impl Hittable) -> Vec<f32> {
        let chunk_size = 4096;
        let pixel_count = self.image_height * self.image_width;
        let pixels: Vec<_> = (0..pixel_count).collect();
        pixels
            .par_chunks(chunk_size)
            .flat_map(|chunk| {
                let mut pixels = vec![];
                for index in chunk {
                    let i = index % self.image_width;
                    let j = index / self.image_width;
                    let mut color = Vec3::zero();
                    for _ in 0..self.samples_per_pixel {
                        let ray = self.get_ray(i, j);
                        color += self.ray_color(&ray, self.max_depth, world);
                    }
                    let result = color * pixel_samples_scale;
                    pixels.extend([result.x, result.y, result.z]);
                }

                pixels
            })
            .collect()
    }

    fn render_sequential(&self, pixel_samples_scale: f32, world: &impl Hittable) -> Vec<f32> {
        let mut pixels = Vec::with_capacity(self.image_height * self.image_width * 3);
        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let mut color = Vec3::zero();
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    color += self.ray_color(&ray, self.max_depth, world);
                }
                let result = color * pixel_samples_scale;
                pixels.push(linear_to_gamma(result.x));
                pixels.push(linear_to_gamma(result.y));
                pixels.push(linear_to_gamma(result.z));
            }
        }
        pixels
    }

    pub fn render(&self, world: &impl Hittable, mode: RenderMode) -> RgbImage {
        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f32;

        let start = Instant::now();
        let pixels = match mode {
            RenderMode::Parallel => self.render_parallel(pixel_samples_scale, world),
            RenderMode::Sequential => self.render_sequential(pixel_samples_scale, world),
            RenderMode::ParallelChunks => self.render_parallel_chunks(pixel_samples_scale, world),
        };
        let duration = start.elapsed();

        info!("rendering took {duration:?}");
        let image =
            Rgb32FImage::from_vec(self.image_width as u32, self.image_height as u32, pixels)
                .expect("dimensions must match");

        DynamicImage::from(image).into_rgb8()
    }
}
