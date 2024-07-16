use std::path::{Path, PathBuf};
use std::time::Instant;

use gltf::Image;
use image::{DynamicImage, Rgb32FImage, RgbImage};
use indicatif::ProgressIterator;
use rayon::prelude::*;
use tev_client::{PacketCreateImage, PacketUpdateImage, TevClient};
use tracing::{debug, info};

use crate::camera::Camera;
use crate::material::{ScatterResult, Scatterable};
use crate::object::Hittable;
use crate::range::Range;
use crate::ray::Ray;
use crate::scene::{RenderSettings, SceneDescription};
use crate::util::{measure, try_measure};
use crate::vec3::{Color, Vec3};
use crate::Result;

pub struct Renderer {
    camera: Camera,
    scene: SceneDescription,
    render: RenderSettings,
}

impl Renderer {
    pub fn new(camera: Camera, scene: SceneDescription, render: RenderSettings) -> Self {
        Renderer {
            camera,
            scene,
            render,
        }
    }

    fn ray_color(&self, ray: &Ray, depth: u32, world: &impl Hittable) -> Color {
        if depth == 0 {
            return Vec3::ZERO;
        }

        let intersection = world.hit(ray, Range::new(self.camera.z_near, self.camera.z_far));
        match intersection {
            Some(hit) => {
                let emitted_color = hit.material.emit(hit.tex_coords, hit.point);

                match hit.material.scatter(ray, &hit) {
                    Some(ScatterResult {
                        attenuation,
                        scattered,
                    }) => {
                        let scattering_pdf = hit.material.scattering_pdf(ray, &hit, &scattered);
                        let pdf = scattering_pdf;
                        debug!("pdf: {:?}", pdf);
                        let scattered_color = if let Some(pdf) = pdf {
                            (attenuation * pdf * self.ray_color(&scattered, depth - 1, world)) / pdf
                        } else {
                            attenuation * self.ray_color(&scattered, depth - 1, world)
                        };
                        emitted_color + scattered_color
                    }
                    None => emitted_color,
                }
            }
            None => self.render.background_color,
        }
    }

    pub fn render(&self, square_size: usize) -> RgbImage {
        let start = Instant::now();

        let sample_scale = 1.0 / self.render.samples_per_pixel as f32;
        let width = self.render.image_width;
        let height = self.render.image_height;
        let pixel_count = width * height;
        let pixels: Vec<_> = (0..pixel_count).collect();
        let pixels = pixels
            .par_chunks(square_size * square_size)
            .flat_map(|chunk| {
                let mut pixels = vec![];
                for index in chunk {
                    let i = index % width;
                    let j = index / width;
                    let mut color = Vec3::ZERO;
                    for _ in 0..self.render.samples_per_pixel {
                        let ray = self.camera.get_ray(i, j);
                        color +=
                            self.ray_color(&ray, self.render.max_depth, &self.scene.root_object);
                    }
                    let result = color * sample_scale;
                    pixels.extend([result.x, result.y, result.z]);
                }

                pixels
            })
            .collect();
        let elapsed = start.elapsed();
        info!("Rendering took {elapsed:?}");

        pixels_to_image(pixels, self.render.image_width, self.render.image_height)
    }

    pub fn render_progressive(&self, mut output: ImageOutput, square_size: usize) -> Result<()> {
        let start = Instant::now();
        let pixel_count = self.render.image_height * self.render.image_width;
        let image_size = (pixel_count * 3) as usize;

        let mut aggregate_image = vec![0.0; image_size];

        let sample_count = self.render.samples_per_pixel;
        let chunk_size = square_size * square_size;

        for current_sample in 1..=sample_count {
            let sample_start = Instant::now();
            let sample_scale = 1.0 / current_sample as f32;
            let chunks: Vec<_> = (0..pixel_count).collect();
            let result: Vec<_> = chunks
                .par_chunks(chunk_size)
                .flat_map(|chunk| {
                    let mut pixels = vec![];
                    for index in chunk {
                        let i = index % self.render.image_width;
                        let j = index / self.render.image_width;
                        let ray = self.camera.get_ray(i, j);
                        let color =
                            self.ray_color(&ray, self.render.max_depth, &self.scene.root_object);
                        pixels.extend([color.x, color.y, color.z]);
                    }
                    pixels
                })
                .map(linear_to_gamma)
                .collect();
            assert_eq!(result.len(), image_size, "result length mismatch");
            for (idx, pixel_component) in result.iter().enumerate() {
                aggregate_image[idx] += *pixel_component;
            }
            if should_save_image(current_sample, sample_count) {
                let intermediate_image: Vec<_> =
                    aggregate_image.iter().map(|c| c * sample_scale).collect();

                output.write(
                    intermediate_image,
                    self.render.image_width,
                    self.render.image_height,
                )?;
                info!("Outputted image after sample {}", current_sample);
            }
            let elapsed = sample_start.elapsed();
            info!("Sample {current_sample} took {elapsed:?}");
        }
        let elapsed = start.elapsed();
        info!("Rendering took {elapsed:?}");

        Ok(())
    }
}

fn linear_to_gamma(linear_component: f32) -> f32 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

fn should_save_image(current_sample: u32, sample_count: u32) -> bool {
    let save_interval = 16;

    current_sample <= 5 || current_sample % save_interval == 0 || current_sample == sample_count
}

fn pixels_to_image(pixels: Vec<f32>, width: u32, height: u32) -> RgbImage {
    let image = Rgb32FImage::from_vec(width, height, pixels).expect("dimensions must match");
    DynamicImage::from(image).into_rgb8()
}

#[derive(Debug)]
pub enum ImageOutput {
    File(PathBuf),
    Viewer(TevClient),
}

impl ImageOutput {
    pub fn init(&mut self, width: u32, height: u32) -> Result<()> {
        if let ImageOutput::Viewer(client) = self {
            client.send(PacketCreateImage {
                image_name: "raytracer",
                grab_focus: true,
                width,
                height,
                channel_names: &["R", "G", "B"],
            })?;
        }

        Ok(())
    }

    pub fn write(&mut self, pixels: Vec<f32>, width: u32, height: u32) -> Result<()> {
        match self {
            Self::File(destination) => {
                let image = pixels_to_image(pixels, width, height);
                image.save(&destination)?;
            }
            Self::Viewer(client) => {
                let channel_offsets = &[0, 1, 2];
                let channel_strides = &[3, 3, 3];

                client.send(PacketUpdateImage {
                    image_name: "raytracer",
                    grab_focus: true,
                    width,
                    height,
                    channel_names: &["R", "G", "B"],
                    x: 0,
                    y: 0,
                    data: &pixels,
                    channel_offsets,
                    channel_strides,
                })?;
            }
        }

        Ok(())
    }
}
