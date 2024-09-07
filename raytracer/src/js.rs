use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::{
    color::colorspace::S_RGB,
    math::lerp,
    random::random,
    spectrum::{Blackbody, HasWavelength, SampledWavelengths},
};

#[wasm_bindgen]
pub fn create_spectrum_image(element_id: &str) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(element_id).unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let width = canvas.width();
    let height = canvas.height();

    let w = width as f32;
    let h = height as f32;
    let color_space = &S_RGB;

    let u = random();

    for x in 0..width {
        for _ in 0..height {
            let x_f = x as f32 / w;
            // let y_f = y as f32 / h;

            let temperature = lerp(x_f, 2000.0, 7500.0);
            let spectrum = Blackbody::new(temperature);
            let wavelengths = SampledWavelengths::sample_visible(u);
            let sample = spectrum.sample(&wavelengths);
            let color = sample.to_rgb(wavelengths, color_space);

            // pixels.push((color.r * 255.0) as u8);
            // pixels.push((color.g * 255.0) as u8);
            // pixels.push((color.b * 255.0) as u8);
            // pixels.push(255);
        }
    }
}
