use std::time::Instant;

use clap::ValueEnum;
use image::{DynamicImage, Rgb32FImage, RgbImage};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use tracing::info;

use crate::camera::Camera;
use crate::material::{ScatterResult, Scatterable};
use crate::object::Hittable;
use crate::range::Range;
use crate::ray::Ray;
use crate::scene::SceneDescription;
use crate::vec3::{Color, Vec3};

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum RenderMode {
    Sequential,
    Parallel,
    ParallelChunks,
}

pub struct Renderer {
    camera: Camera,
    scene: SceneDescription,
}

impl Renderer {
    pub fn new(camera: Camera, scene: SceneDescription) -> Self {
        Renderer { camera, scene }
    }

    fn ray_color(&self, ray: &Ray, depth: u32, world: &impl Hittable) -> Color {
        if depth == 0 {
            return Vec3::ZERO;
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
            None => self.scene.background_color,
        }
    }

    fn render_parallel(&self, pixel_samples_scale: f32, world: &impl Hittable) -> Vec<f32> {
        let threads = rayon::current_num_threads();
        let pixel_count = self.scene.image_height * self.scene.image_width;
        info!("rendering in parallel with {threads} threads");
        (0..pixel_count)
            .into_par_iter()
            .progress_count(pixel_count as u64)
            .flat_map(|index| {
                let i = index % self.scene.image_width;
                let j = index / self.scene.image_width;
                let mut color = Vec3::ZERO;
                for _ in 0..self.scene.samples_per_pixel {
                    let ray = self.camera.get_ray(i, j);
                    color += self.ray_color(&ray, self.scene.max_depth, world);
                }
                let result = color * pixel_samples_scale;
                [result.x, result.y, result.z]
            })
            .map(linear_to_gamma)
            .collect()
    }

    fn render_parallel_chunks(&self, pixel_samples_scale: f32, world: &impl Hittable) -> Vec<f32> {
        let chunk_size = 4096;
        let pixel_count = self.scene.image_height * self.scene.image_width;
        let pixels: Vec<_> = (0..pixel_count).collect();
        pixels
            .par_chunks(chunk_size)
            .flat_map(|chunk| {
                let mut pixels = vec![];
                for index in chunk {
                    let i = index % self.scene.image_width;
                    let j = index / self.scene.image_width;
                    let mut color = Vec3::ZERO;
                    for _ in 0..self.scene.samples_per_pixel {
                        let ray = self.camera.get_ray(i, j);
                        color += self.ray_color(&ray, self.scene.max_depth, world);
                    }
                    let result = color * pixel_samples_scale;
                    pixels.extend([result.x, result.y, result.z]);
                }

                pixels
            })
            .collect()
    }

    fn render_sequential(&self, pixel_samples_scale: f32, world: &impl Hittable) -> Vec<f32> {
        let mut pixels =
            Vec::with_capacity((self.scene.image_height * self.scene.image_width * 3) as usize);
        for j in 0..self.scene.image_height {
            for i in 0..self.scene.image_width {
                let mut color = Vec3::ZERO;
                for _ in 0..self.scene.samples_per_pixel {
                    let ray = self.camera.get_ray(i, j);
                    color += self.ray_color(&ray, self.scene.max_depth, world);
                }
                let result = color * pixel_samples_scale;
                pixels.push(linear_to_gamma(result.x));
                pixels.push(linear_to_gamma(result.y));
                pixels.push(linear_to_gamma(result.z));
            }
        }
        pixels
    }

    pub fn render(&self, mode: RenderMode) -> RgbImage {
        let pixel_samples_scale = 1.0 / self.scene.samples_per_pixel as f32;

        let start = Instant::now();
        let pixels = match mode {
            RenderMode::Parallel => {
                self.render_parallel(pixel_samples_scale, &self.scene.root_object)
            }
            RenderMode::Sequential => {
                self.render_sequential(pixel_samples_scale, &self.scene.root_object)
            }
            RenderMode::ParallelChunks => {
                self.render_parallel_chunks(pixel_samples_scale, &self.scene.root_object)
            }
        };
        let duration = start.elapsed();

        info!("rendering took {duration:?}");
        let image = Rgb32FImage::from_vec(self.scene.image_width, self.scene.image_height, pixels)
            .expect("dimensions must match");

        DynamicImage::from(image).into_rgb8()
    }
}

fn linear_to_gamma(linear_component: f32) -> f32 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}
