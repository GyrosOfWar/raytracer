use std::{io, io::Write};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug)]
pub struct Image {
    pixels: Vec<Color>,
    width: usize,
    height: usize,
}

impl Image {
    pub fn new(pixels: Vec<Color>, width: usize, height: usize) -> Self {
        Image {
            pixels,
            width,
            height,
        }
    }

    pub fn write_to_ppm(&self, writer: &mut impl Write) -> io::Result<()> {
        write!(writer, "P3\n{} {}\n255\n", self.width, self.height)?;
        for pixel in &self.pixels {
            let r = (pixel.r * 255.999) as u8;
            let g = (pixel.g * 255.999) as u8;
            let b = (pixel.b * 255.999) as u8;
            write!(writer, "{} {} {}\n", r, g, b)?;
        }
        Ok(())
    }
}
