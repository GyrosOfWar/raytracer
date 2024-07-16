use std::fmt::Debug;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

use enum_dispatch::enum_dispatch;
pub use sphere::Sphere;
use tracing::info;
use triangle_mesh::TriangleRef;
pub use world::World;

use crate::aabb::Aabb;
use crate::bvh::{BvhNode, FlatBvhTree};
use crate::material::Material;
use crate::range::Range;
use crate::ray::Ray;
use crate::texture::TextureCoordinates;
use crate::util::measure;
use crate::vec3::{Point3, Vec3};

mod sphere;
pub mod triangle_mesh;
mod world;

pub fn get_id() -> u32 {
    static ID_COUNTER: AtomicU32 = AtomicU32::new(1);

    ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub distance: f32,
    pub front_facing: bool,
    pub material: Arc<Material>,
    pub tex_coords: TextureCoordinates,
}

impl HitRecord {
    pub fn new(
        ray: &Ray,
        outward_normal: Vec3,
        point: Point3,
        distance: f32,
        material: Arc<Material>,
        tex_coords: TextureCoordinates,
    ) -> Self {
        let front_facing = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_facing {
            outward_normal
        } else {
            -outward_normal
        };

        HitRecord {
            point,
            normal,
            front_facing,
            distance,
            material,
            tex_coords,
        }
    }
}

#[enum_dispatch]
pub trait Hittable: Send + Sync + Debug {
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord>;

    fn bounding_box(&self) -> Aabb;

    fn id(&self) -> u32;

    fn name(&self) -> &'static str;
}

#[enum_dispatch(Hittable)]
#[derive(Debug)]
pub enum Object {
    Sphere(Sphere),
    BvhNode(BvhNode),
    FlatBvhTree(FlatBvhTree),
    World(World),
    TriangleRef(TriangleRef),
}

impl Object {
    pub fn len(&self) -> usize {
        match self {
            Object::Sphere(_) => 1,
            Object::BvhNode(node) => node.len(),
            Object::FlatBvhTree(tree) => tree.len(),
            Object::World(world) => world.objects.len(),
            Object::TriangleRef(_) => 1,
        }
    }
}
