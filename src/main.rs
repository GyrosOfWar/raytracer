use std::{error::Error, path::PathBuf, sync::Arc};

use bvh::BvhNode;
use camera::{Camera, CameraParams, RenderMode};
use clap::Parser;
use object::{triangle_mesh, Hittable, Object};
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
mod texture;
mod vec3;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    pub bvh_disabled: bool,

    #[clap(short, long, default_value = "parallel")]
    pub render_mode: RenderMode,

    #[clap(long)]
    pub debug: bool,

    pub input: PathBuf,

    #[clap(default_value = "image.jpeg")]
    pub output: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let args = Args::parse();

    let meshes = triangle_mesh::load_from_gltf(&args.input)?;
    info!("rendering with configuration {args:#?}");
    let world = Object::BvhNode(BvhNode::from(meshes));
    info!("extents of the scene: {:?}", world.bounding_box());

    let zoom = 100.0;

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
            let root = Arc::new(world);
            bvh::debug::validate_tree(root.clone());
            bvh::debug::print_tree(root, 0);
        }
    } else {
        let image = camera.render(&world, args.render_mode);
        image.save(args.output)?;
    }

    Ok(())
}
