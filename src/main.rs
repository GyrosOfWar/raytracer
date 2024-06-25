use std::{fs::File, io};

use ppm::{Color, Image};

mod ppm;
mod ray;
mod vec3;

fn main() -> io::Result<()> {
    let height = 256;
    let width = 256;

    let mut pixels = vec![];
    for j in 0..height {
        for i in 0..width {
            let r = i as f32 / (width - 1) as f32;
            let g = j as f32 / (height - 1) as f32;
            let b = 0.0;
            pixels.push(Color { r, g, b });
        }
    }

    let image = Image::new(pixels, width, height);
    let mut file = File::create("image.ppm")?;
    image.write_to_ppm(&mut file)?;
    Ok(())
}
