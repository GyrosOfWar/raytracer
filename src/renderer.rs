use std::path::Path;

use clap::ValueEnum;
use image::{DynamicImage, Rgb32FImage, RgbImage};
use indicatif::ProgressIterator;
use rayon::prelude::*;
use tracing::info;

use crate::camera::Camera;
use crate::material::{ScatterResult, Scatterable};
use crate::object::Hittable;
use crate::range::Range;
use crate::ray::Ray;
use crate::scene::SceneDescription;
use crate::vec3::{Color, Vec3};
use crate::Result;

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

    pub fn render_progressive(
        &self,
        destination: impl AsRef<Path>,
        chunk_size_sqrt: usize,
    ) -> Result<()> {
        let pixel_count = self.scene.render.image_height * self.scene.render.image_width;
        let mut aggregate_image = vec![0.0; (pixel_count * 3) as usize];

        let sample_count = self.scene.render.samples_per_pixel;
        let chunk_size = chunk_size_sqrt * chunk_size_sqrt;

        // TODO variable step size like in pbrt
        for current_sample in (1..=sample_count).progress_count(sample_count as u64) {
            let sample_scale = 1.0 / current_sample as f32;
            let pixels: Vec<_> = (0..pixel_count).collect();
            let result: Vec<_> = pixels
                .par_chunks(chunk_size)
                .flat_map(|chunk| {
                    let mut pixels = vec![];
                    for index in chunk {
                        let i = index % self.scene.render.image_width;
                        let j = index / self.scene.render.image_width;
                        let ray = self.camera.get_ray(i, j);
                        let color = self.ray_color(
                            &ray,
                            self.scene.render.max_depth,
                            &self.scene.root_object,
                        );
                        pixels.extend([color.x, color.y, color.z]);
                    }
                    pixels
                })
                .map(linear_to_gamma)
                .collect();

            for (idx, pixel_component) in result.iter().enumerate() {
                aggregate_image[idx] += *pixel_component;
            }

            let intermediate_image: Vec<_> =
                aggregate_image.iter().map(|c| c * sample_scale).collect();
            let image = self.pixels_to_image(intermediate_image);
            image.save(&destination)?;
        }

        Ok(())
    }

    fn pixels_to_image(&self, pixels: Vec<f32>) -> RgbImage {
        let image = Rgb32FImage::from_vec(
            self.scene.render.image_width,
            self.scene.render.image_height,
            pixels,
        )
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
