use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::{
    aabb::Aabb,
    bvh::BvhNode,
    material::Material,
    range::Range,
    ray::Ray,
    texture::TextureCoordinates,
    vec3::{Point3, Vec3},
};
use enum_dispatch::enum_dispatch;
pub use sphere::Sphere;
use triangle_mesh::TriangleRef;
pub use world::World;

mod sphere;
pub mod triangle_mesh;
mod world;

pub fn get_id() -> u32 {
    static ID_COUNTER: AtomicU32 = AtomicU32::new(1);

    ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub struct HitRecord {
    pub point: Point3<f32>,
    pub normal: Vec3<f32>,
    pub distance: f32,
    pub front_facing: bool,
    pub material: Arc<Material>,
    pub tex_coords: TextureCoordinates,
}

impl HitRecord {
    pub fn new(
        ray: &Ray,
        outward_normal: Vec3<f32>,
        point: Point3<f32>,
        distance: f32,
        material: Arc<Material>,
        tex_coords: TextureCoordinates,
    ) -> Self {
        let front_facing = ray.direction.dot(&outward_normal) < 0.0;
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
pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord>;

    fn bounding_box(&self) -> Aabb;

    fn id(&self) -> u32;
}

#[enum_dispatch(Hittable)]
#[derive(Debug)]
pub enum Object {
    Sphere(Sphere),
    BvhNode(BvhNode),
    World(World),
    TriangleRef(TriangleRef),
}

#[cfg(test)]
mod tests {
    use crate::{
        material::helpers::metal,
        object::Hittable,
        vec3::{Point3, Vec3},
    };

    use super::Sphere;

    #[test]
    fn get_sphere_bbox() {
        let material = metal(Vec3::new(0.5, 0.1, 0.7), 0.2);
        let sphere = Sphere::new(Point3::new(0.0, 0.0, 0.0), 2.0, material);
        let bbox = sphere.bounding_box();
        assert_eq!(bbox.x.min, -2.0);
        assert_eq!(bbox.y.min, -2.0);
        assert_eq!(bbox.z.min, -2.0);
        assert_eq!(bbox.x.max, 2.0);
        assert_eq!(bbox.y.max, 2.0);
        assert_eq!(bbox.z.max, 2.0);
    }
}
