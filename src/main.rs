use bvh::BvhNode;
use object::World;

mod aabb;
mod bvh;
mod camera;
mod helpers;
mod material;
mod object;
mod range;
mod ray;
mod scenes;
mod texture;
mod vec3;

const DEBUG_BVH: bool = false;

fn main() -> Result<(), image::ImageError> {
    let arg = std::env::args().nth(1).unwrap_or("spheres".into());

    let (camera, objects) = match arg.as_str() {
        "spheres" => scenes::lots_of_spheres(),
        "earth" => scenes::earth(),
        _ => panic!("unknown scene"),
    };

    let world = World::new(objects);
    let world = BvhNode::from_world(world);

    let image = camera.render(&world);
    let file_name = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "image.jpeg".into());
    image.save(file_name)?;

    Ok(())
}
