use std::path::PathBuf;

use clap::Parser;
use image::{Pixel, Rgba};
use mimalloc::MiMalloc;
use pixels::wgpu::Color;
use pixels::{Pixels, SurfaceTexture};
use raytracer::color::colorspace::{RgbColorSpace, S_RGB};
use raytracer::color::rgb::Rgb;
use raytracer::math::lerp;
use raytracer::random::random;
use raytracer::sample::stratified_1d;
use raytracer::spectrum::{
    Blackbody, Constant, HasWavelength, RgbAlbedo, SampledWavelengths, Spectrum, LAMBDA_MAX,
    LAMBDA_MIN,
};
use tracing::{error, info, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long)]
    pub bvh_disabled: bool,

    #[clap(long)]
    pub debug: bool,

    #[clap(short = 'W', long, default_value = "1280")]
    pub width: u32,

    #[clap(short = 'H', long, default_value = "720")]
    pub height: u32,

    #[clap(short = 'd', long, default_value = "50")]
    pub max_depth: u32,

    #[clap(long = "spp", default_value = "100")]
    pub samples_per_pixel: u32,

    #[clap(short, long, default_value = "0")]
    pub camera: usize,

    pub input: PathBuf,

    #[clap(default_value = "image.jpeg")]
    pub output: PathBuf,
}

fn color_to_slice(color: Rgb) -> [u8; 4] {
    let r = (color.r * 255.0).round() as u8;
    let g = (color.g * 255.0).round() as u8;
    let b = (color.b * 255.0).round() as u8;
    [r, g, b, 255]
}

fn create_spectrum_image(w: u32, h: u32) -> Vec<u8> {
    use image::RgbaImage;

    let cs = &S_RGB;

    let mut image = RgbaImage::new(w, h);
    for (x, _, pixel) in image.enumerate_pixels_mut() {
        let x_f = x as f32 / w as f32;
        let temp = lerp(x_f, 1500.0, 9000.0);
        let spectrum = Blackbody::new(temp);
        let mut color = Rgb::ZERO;
        for _ in 0..100 {
            let u = random();
            let wavelengths = SampledWavelengths::sample_visible(u);
            let sample = spectrum.sample(&wavelengths);
            color += sample.to_rgb(wavelengths, cs);
        }

        color = color / 100.0;

        pixel.0 = [
            (color.r * 255.0).round() as u8,
            (color.g * 255.0).round() as u8,
            (color.b * 255.0).round() as u8,
            255,
        ];
    }

    image.into_vec()
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(if args.debug {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .init();

    let event_loop = EventLoop::new();
    let input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("raytracer")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let image = create_spectrum_image(WIDTH, HEIGHT);

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let frame = pixels.frame_mut();
            frame.copy_from_slice(&image);

            if let Err(err) = pixels.render() {
                error!("failed to render frame: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
            *control_flow = ControlFlow::Exit;
            return;
        }

        // Resize the window
        if let Some(size) = input.window_resized() {
            if let Err(err) = pixels.resize_surface(size.width, size.height) {
                tracing::error!("pixels.resize_surface: {}", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        window.request_redraw();
    });
}
