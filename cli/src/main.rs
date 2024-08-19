use std::path::PathBuf;

use clap::Parser;
use mimalloc::MiMalloc;
use pixels::wgpu::Color;
use pixels::{Pixels, SurfaceTexture};
use raytracer::color::colorspace::{RgbColorSpace, S_RGB};
use raytracer::color::rgb::Rgb;
use raytracer::math::lerp;
use raytracer::random::random;
use raytracer::spectrum::{
    Constant, HasWavelength, RgbAlbedo, SampledWavelengths, Spectrum, LAMBDA_MAX, LAMBDA_MIN,
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

fn remap_range(value: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
    (value - old_min) / (old_max - old_min) * (new_max - new_min) + new_min
}

fn render_spectrum(color_space: &RgbColorSpace, buffer: &mut [u8]) {
    let start = std::time::Instant::now();
    for (i, pixel) in buffer.chunks_exact_mut(4).enumerate() {
        let x = i / WIDTH as usize;
        let y = i % WIDTH as usize;
        let lambda = remap_range(x as f32, 0.0, WIDTH as f32, LAMBDA_MIN, LAMBDA_MAX);

        let spectrum = Spectrum::Constant(Constant { c: lambda });
        let mut color = Rgb::ZERO;
        let n = 10;
        for _ in 0..n {
            let u = random();
            let lambda = SampledWavelengths::sample_uniform(u);
            let sample = spectrum.sample(&lambda);
            let rgb = sample.to_rgb(lambda, color_space);
            color += rgb;
        }
        let color = color / n as f32;

        pixel.copy_from_slice(&color_to_slice(color));
        info!("pixel result: {:?}, color = {:?}", pixel, color);
    }
    let elapsed = start.elapsed();
    info!("rendered frame in {:?}", elapsed);
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

    let color_space = &S_RGB;

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let frame = pixels.frame_mut();
            render_spectrum(color_space, frame);

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
