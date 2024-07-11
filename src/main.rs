use std::path::PathBuf;

use camera::Camera;
use clap::Parser;
use mimalloc::MiMalloc;
use object::Hittable;
use renderer::Renderer;
use scene::RenderSettings;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

mod aabb;
mod bvh;
mod camera;
mod material;
mod object;
mod random;
mod range;
mod ray;
mod renderer;
mod scene;
mod texture;
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

    #[clap(short, long, default_value = "1280")]
    pub width: u32,

    #[clap(short, long, default_value = "720")]
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

    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let args = Args::parse();

    let render_settings = RenderSettings {
        samples_per_pixel: args.samples_per_pixel,
        selected_camera: args.camera,
        image_width: args.width,
        image_height: args.height,
        max_depth: args.max_depth,
    };

    let selected_camera = render_settings.selected_camera;

    let scene = scene::load_from_gltf(&args.input, args.bvh_disabled, render_settings)?;
    info!(
        "extents of the scene: {:#?}",
        scene.root_object.bounding_box()
    );
    info!("rendering with configuration {args:#?}");

    let camera = Camera::new(scene.camera(selected_camera), args.width, args.height);

    let renderer = Renderer::new(camera, scene);
    renderer.render_progressive(args.output, 256)?;
    // let image = renderer.render(256);
    // image.save(&args.output)?;

    Ok(())
}
