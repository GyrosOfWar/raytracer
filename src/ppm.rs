use std::{io, io::Write};

use num_traits::Float;

use crate::{trace::Range, vec3::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl<T: Float> From<Vec3<T>> for Color {
    fn from(value: Vec3<T>) -> Self {
        Color {
            r: num_traits::cast(value.x).unwrap_or(0.0),
            g: num_traits::cast(value.y).unwrap_or(0.0),
            b: num_traits::cast(value.z).unwrap_or(0.0),
        }
    }
}

#[derive(Debug)]
pub struct Image {
    pixels: Vec<Color>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn new(pixels: Vec<Color>, width: usize, height: usize) -> Self {
        if pixels.len() != (width * height) {
            panic!(
                "mismatching image dimensions: expected {width} * {height} = {} pixels, got {}",
                (width * height),
                pixels.len()
            );
        }

        Image {
            pixels,
            width,
            height,
        }
    }

    pub fn write_to_ppm(&self, writer: &mut impl Write) -> io::Result<()> {
        let intensity = Range::new(0.0, 0.999);
        write!(writer, "P3\n{} {}\n255\n", self.width, self.height)?;
        for pixel in &self.pixels {
            let r = (256.0 * intensity.clamp(pixel.r)) as u32;
            let g = (256.0 * intensity.clamp(pixel.g)) as u32;
            let b = (256.0 * intensity.clamp(pixel.b)) as u32;
            write!(writer, "{} {} {}\n", r, g, b)?;
        }
        Ok(())
    }
}
