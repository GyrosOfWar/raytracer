use std::sync::Arc;

use tracing::info;

use crate::{
    aabb::Aabb,
    material::Material,
    object::get_id,
    range::Range,
    ray::Ray,
    texture::TextureCoordinates,
    vec3::{Point3, Vec3},
};

use super::{HitRecord, Hittable};

#[derive(Debug)]
pub struct Quad {
    /// Origin point
    q: Point3<f32>,
    /// First direction vector
    v: Vec3<f32>,
    /// Second direction vector
    u: Vec3<f32>,
    id: u32,
    material: Arc<Material>,
    bbox: Aabb,
    normal: Vec3<f32>,
    d: f32,
    w: Vec3<f32>,
}

impl Quad {
    pub fn new(q: Point3<f32>, u: Vec3<f32>, v: Vec3<f32>, material: Arc<Material>) -> Self {
        let diagonal1 = Aabb::from_points(q, q + u + v);
        let diagonal2 = Aabb::from_points(q + u, q + v);
        let bbox = Aabb::from_boxes(diagonal1, diagonal2);
        let id = get_id();
        let normal = u.cross(v).unit();
        let d = normal.dot(q);
        let w = normal / normal.dot(normal);
        info!("q = {q:?}, u = {u:?}, v = {v:?}, normal = {normal:?}, d = {d}, w = {w:?}");

        Quad {
            q,
            v,
            u,
            id,
            material,
            bbox,
            normal,
            d,
            w,
        }
    }
}

fn is_interior(a: f32, b: f32) -> Option<TextureCoordinates> {
    if Range::UNIT.contains(a) && Range::UNIT.contains(b) {
        Some(TextureCoordinates { u: a, v: b })
    } else {
        None
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        let denom = self.normal.dot(ray.direction);

        // ray is parallel to the plane
        if denom.abs() < 1e-4 {
            return None;
        }

        let t = (self.d - Vec3::dot(self.normal, ray.origin)) / denom;
        if !hit_range.contains(t) {
            return None;
        }

        let intersection = ray.evaluate(t);
        let planar_hitpt_vector = intersection - self.q;

        let alpha = Vec3::dot(self.w, Vec3::cross(planar_hitpt_vector, self.v));
        let beta = Vec3::dot(self.w, Vec3::cross(self.u, planar_hitpt_vector));
        // info!("alpha: {}, beta: {}", alpha, beta);

        // Some(HitRecord::new(
        //     ray,
        //     self.normal,
        //     intersection,
        //     t,
        //     self.material.clone(),
        //     Default::default(),
        // ))

        is_interior(alpha, beta).map(|tex_coords| {
            HitRecord::new(
                ray,
                self.normal,
                intersection,
                t,
                self.material.clone(),
                tex_coords,
            )
        })
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn id(&self) -> u32 {
        self.id
    }
}
