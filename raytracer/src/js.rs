use wasm_bindgen::prelude::*;

use crate::color::colorspace::S_RGB;
use crate::math::lerp;
use crate::random::random;
use crate::spectrum::{Blackbody, HasWavelength, SampledWavelengths};

#[wasm_bindgen]
pub fn create_spectrum_image(width: u32, height: u32) -> Vec<f32> {
    console_error_panic_hook::set_once();

    let w = width as f32;
    let h = height as f32;
    let color_space = &S_RGB;

    let u = random();

    let mut pixels = Vec::new();
    for x in 0..width {
        for y in 0..height {
            let x_f = x as f32 / w;
            let y_f = y as f32 / h;

            let temperature = lerp(x_f, 2000.0, 7500.0);
            let spectrum = Blackbody::new(temperature);
            let wavelengths = SampledWavelengths::sample_visible(u);
            let sample = spectrum.sample(&wavelengths);
            let color = sample.to_rgb(wavelengths, color_space);

            pixels.push(color.r);
            pixels.push(color.g);
            pixels.push(color.b);
            pixels.push(1.0);
        }
    }

    pixels
}
