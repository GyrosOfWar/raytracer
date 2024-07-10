use std::path::PathBuf;
use std::sync::Arc;

use camera::Camera;
use clap::Parser;
use object::Hittable;
use renderer::{RenderMode, Renderer};
use scene::RenderSettings;
use tracing::{error, info};
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
    let width = 1280;
    let height = 720;

    let render_settings = RenderSettings {
        samples_per_pixel: 100,
        selected_camera: 0,
        image_width: width,
        image_height: height,
        max_depth: 50,
    };

    let scene = scene::load_from_gltf(&args.input, args.bvh_disabled, render_settings)?;
    info!(
        "extents of the scene: {:?}",
        scene.root_object.bounding_box()
    );
    info!("rendering with configuration {args:#?}");

    let camera = Camera::new(
        scene.cameras[scene.render.selected_camera].clone(),
        width,
        height,
    );

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
        renderer.render_progressive(args.output, 256)?;
    }

    Ok(())
}
