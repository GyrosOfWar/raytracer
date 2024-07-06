use std::{error::Error, fs::File, sync::Arc};

use bvh::BvhNode;
use camera::RenderMode;
use object::{Object, Scene, World};
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
mod scenes;
mod texture;
mod vec3;

#[derive(Debug)]
pub struct Configuration {
    pub bvh_disabled: bool,
    pub sequential_rendering: bool,
    pub bvh_debug: bool,
}

impl Configuration {
    pub fn from_env() -> Configuration {
        let bvh_disabled = std::env::var("RT_BVH_DISABLED").is_ok();
        let sequential_rendering = std::env::var("RT_SEQUENTIAL").is_ok();
        let bvh_debug = std::env::var("RT_DEBUG").is_ok();

        Configuration {
            bvh_disabled,
            sequential_rendering,
            bvh_debug,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let scene = std::env::args().nth(1).unwrap_or("quads".into());
    info!("rendering scene '{scene}'");
    let file = File::open(scene)?;

    let scene: Scene = serde_json::from_reader(file)?;
    let (camera, objects) = scene.build();

    let config = Configuration::from_env();
    info!("rendering with configuration {config:#?}");
    let world = if config.bvh_disabled {
        Object::World(World::new(objects))
    } else {
        Object::BvhNode(BvhNode::from_world(objects))
    };

    if config.bvh_debug {
        if config.bvh_disabled {
            error!("BVH is disabled, nothing to show.");
        } else {
            let root = Arc::new(world);
            bvh::debug::validate_tree(root.clone());
            bvh::debug::print_tree(root, 0);
        }
    } else {
        let mode = if config.sequential_rendering {
            RenderMode::Sequential
        } else {
            RenderMode::Parallel
        };
        let image = camera.render(&world, mode);
        let file_name = std::env::args()
            .nth(2)
            .unwrap_or_else(|| "image.jpeg".into());
        image.save(file_name)?;
    }

    Ok(())
}
