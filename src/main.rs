use std::path::PathBuf;
use std::sync::Arc;

use camera::{Camera, CameraParams};
use clap::Parser;
use object::Hittable;
use renderer::{RenderMode, Renderer};
use tracing::{error, info};
use tracing_subscriber::fmt::format::FmtSpan;
use vec3::{Color, Point3};

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

pub type Result<T> = color_eyre::Result<T>;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long)]
    pub bvh_disabled: bool,

    #[clap(short, long, default_value = "parallel")]
    pub render_mode: RenderMode,

    #[clap(long)]
    pub debug: bool,

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

    let scene = scene::load_from_gltf(&args.input, args.bvh_disabled)?;
    info!(
        "extents of the scene: {:?}",
        scene.root_object.bounding_box()
    );
    info!("rendering with configuration {args:#?}");

    let zoom = 0.01;

    let camera = Camera::new(CameraParams {
        look_from: Point3::new(2.0 * zoom, 1.5 * zoom, -3.0 * zoom),
        background_color: Color::new(0.5, 0.5, 0.5),
        samples_per_pixel: 100,
        ..Default::default()
    });

    if args.debug {
        if args.bvh_disabled {
            error!("BVH is disabled, nothing to show.");
        } else {
            let root = Arc::new(scene.root_object);
            bvh::debug::validate_tree(root.clone());
            bvh::debug::print_tree(root, 0);
        }
    } else {
        let renderer = Renderer::new(camera, scene);
        let image = renderer.render(args.render_mode);
        image.save(args.output)?;
    }

    Ok(())
}
