#![allow(unused)]
use std::path::PathBuf;

use bvh::BvhType;
use camera::Camera;
use clap::Parser;
use mimalloc::MiMalloc;
use object::Hittable;
use renderer::{ImageOutput, Renderer};
use scene::RenderSettings;
use tev_client::TevClient;
use tracing::{info, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use vec3::Color;

mod aabb;
mod bvh;
mod camera;
mod material;
mod math;
mod object;
mod onb;
mod random;
mod range;
mod ray;
mod renderer;
mod sample;
mod scene;
mod texture;
mod util;
mod vec3;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub type Result<T> = color_eyre::Result<T>;

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

fn main() -> Result<()> {
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

    let render_settings = RenderSettings {
        samples_per_pixel: args.samples_per_pixel,
        selected_camera: args.camera,
        image_width: args.width,
        image_height: args.height,
        max_depth: args.max_depth,
        background_color: Color::ZERO,
    };

    let selected_camera = render_settings.selected_camera;

    let scene = scene::load_from_gltf(&args.input)?.build_bvh(BvhType::Tree);
    info!(
        "extents of the scene: {:#?}",
        scene.root_object.bounding_box()
    );
    info!("rendering with configuration {args:#?}");

    let mut output = ImageOutput::Viewer(TevClient::spawn_path_default()?);
    output.init(render_settings.image_width, render_settings.image_height)?;

    let camera = Camera::new(scene.camera(selected_camera), args.width, args.height);
    let renderer = Renderer::new(camera, scene, render_settings);

    renderer.render_progressive(output, 16)?;
    Ok(())
}
