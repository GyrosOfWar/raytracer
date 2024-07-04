use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::{
    aabb::Aabb,
    bvh::BvhNode,
    camera::CameraParams,
    material::{Material, MaterialBuilder},
    range::Range,
    ray::Ray,
    texture::TextureCoordinates,
    vec3::{Point3, Vec3},
};
use enum_dispatch::enum_dispatch;
pub use quad::Quad;
use serde::{Deserialize, Serialize};
pub use sphere::Sphere;
pub use world::World;

mod quad;
mod sphere;
mod triangle_mesh;
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
        ray: &Ray<f32>,
        outward_normal: Vec3<f32>,
        point: Point3<f32>,
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
pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord>;

    fn bounding_box(&self) -> Aabb;

    fn id(&self) -> u32;
}

#[enum_dispatch(Hittable)]
#[derive(Debug)]
pub enum Object {
    Sphere(Sphere),
    BvhNode(BvhNode),
    Quad(Quad),
    World(World),
}

#[derive(Debug)]
pub struct Scene {
    pub objects: Vec<ObjectBuilder>,
    pub camera: CameraParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ObjectBuilder {
    Sphere(SphereBuilder),
    Quad(QuadBuilder),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SphereBuilder {
    pub radius: f32,
    pub center: Point3<f32>,
    pub material: MaterialBuilder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuadBuilder {
    pub q: Point3<f32>,
    pub u: Vec3<f32>,
    pub v: Vec3<f32>,
    pub material: MaterialBuilder,
}

impl From<ObjectBuilder> for Object {
    fn from(value: ObjectBuilder) -> Self {
        match value {
            ObjectBuilder::Sphere(sphere) => Object::Sphere(Sphere::new(
                sphere.center,
                sphere.radius,
                Arc::new(sphere.material.into()),
            )),
            ObjectBuilder::Quad(quad) => Object::Quad(Quad::new(
                quad.q,
                quad.u,
                quad.v,
                Arc::new(quad.material.into()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{material::metal, object::Hittable, vec3::Point3};

    use super::Sphere;

    #[test]
    fn get_sphere_bbox() {
        let material = metal(Point3::new(0.5, 0.1, 0.7), 0.2);
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
