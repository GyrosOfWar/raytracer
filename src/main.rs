use std::{fs::File, io};

use num_traits::Zero;
use ppm::{Color, Image};
use ray::Ray;
use vec3::{Point3, Vec3};

mod ppm;
mod ray;
mod vec3;

fn ray_color(ray: &Ray<f32>) -> Color {
    Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    }
}

fn main() -> io::Result<()> {
    let aspect_ratio = 16.0 / 9.0;
    let width = 400;
    let height = ((width as f64 / aspect_ratio) as usize).max(1);

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
    let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

    let mut pixels = vec![];
    for j in 0..height {
        for i in 0..width {
            let pixel_center =
                pixel00_loc + (pixel_delta_u * i as f32) + (pixel_delta_v * j as f32);
            let ray_direction = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_direction);

            let color = ray_color(&ray);
            pixels.push(color);
        }
    }

    let image = Image::new(pixels, width, height);
    let mut file = File::create("image.ppm")?;
    image.write_to_ppm(&mut file)?;
    Ok(())
}
