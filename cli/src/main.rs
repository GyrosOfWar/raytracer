use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Instant;

use clap::Parser;
use color_eyre::Result;
use image::{DynamicImage, Rgba32FImage, RgbaImage};
use mimalloc::MiMalloc;
use pixels::{Pixels, SurfaceTexture};
use raytracer::color::colorspace::DCI_P3;
use raytracer::color::rgb::Rgb;
use raytracer::random::random;
use raytracer::spectrum::{HasWavelength, RgbAlbedo, SampledWavelengths};
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

fn create_spectrum_image(image: &mut Rgba32FImage) {
    let cs = &DCI_P3;
    let w = image.width() as f32;
    let h = image.height() as f32;

    let start = Instant::now();
    let u = random();
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let x_f = x as f32 / w;
        let y_f = y as f32 / h;
        let spectrum = RgbAlbedo::with_color_space(
            cs,
            Rgb {
                r: x_f,
                g: 0.0,
                b: y_f,
            },
        );
        let wavelengths = SampledWavelengths::sample_visible(u);
        let sample = spectrum.sample(&wavelengths);
        let color = sample.to_rgb(wavelengths, cs);
        pixel.0[0] += color.r;
        pixel.0[1] += color.g;
        pixel.0[2] += color.b;
        pixel.0[3] = 1.0;
    }
    let elapsed = start.elapsed();
    info!("took {elapsed:?} to make image");
}

fn save_image(image: &Rgba32FImage) {
    let image = DynamicImage::ImageRgba32F(image.clone());

    rayon::spawn(move || {
        let image = image.into_rgba8();
        if let Err(e) = image.save("image.png") {
            error!("failed to save image: {e}");
        }
    });
}

fn render_image(rx: Receiver<RgbaImage>) -> Result<()> {
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

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            if let Ok(image) = rx.try_recv() {
                let frame = pixels.frame_mut();
                frame.copy_from_slice(&image);
            }

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

    let (rx, tx) = channel();
    thread::spawn(move || {
        let mut buffer = Rgba32FImage::new(WIDTH, HEIGHT);
        for sample in 1.. {
            info!("sample: {sample}");
            create_spectrum_image(&mut buffer);
            let mut image = buffer.clone();
            for (_, _, pixel) in image.enumerate_pixels_mut() {
                let n = sample as f32;
                pixel.0[0] /= n;
                pixel.0[1] /= n;
                pixel.0[2] /= n;
            }

            if sample % 50 == 0 {
                save_image(&image);
            }

            rx.send(DynamicImage::ImageRgba32F(image).into_rgba8())
                .expect("can't send");
        }
    });
    render_image(tx)?;

    Ok(())
}
