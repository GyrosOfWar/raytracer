use std::sync::Arc;

use bvh::BvhNode;
use object::Object;

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

fn main() -> Result<(), image::ImageError> {
    let arg = std::env::args().nth(1).unwrap_or("spheres".into());

    let (camera, objects) = match arg.as_str() {
        "spheres" => scenes::lots_of_spheres(),
        "earth" => scenes::earth(),
        "quads" => scenes::quads(),
        _ => scenes::quads(),
    };

    let world = BvhNode::from_world(objects);
    let debug = std::env::var("RT_DEBUG").is_ok();

    if debug {
        let root = Arc::new(Object::BvhNode(world));
        bvh::debug::validate_tree(root.clone());
        bvh::debug::print_tree(root, 0);
    } else {
        let image = camera.render(&world);
        let file_name = std::env::args()
            .nth(2)
            .unwrap_or_else(|| "image.jpeg".into());
        image.save(file_name)?;
    }

    Ok(())
}
