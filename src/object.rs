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

static ID_COUNTER: AtomicU32 = AtomicU32::new(1);

pub fn get_id() -> u32 {
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
}

#[derive(Debug)]
pub struct World {
    objects: Vec<Object>,
    bounding_box: Aabb,
}

fn make_bounding_box(objects: &[Object]) -> Aabb {
    let mut bbox = Aabb::EMPTY;
    for object in objects {
        bbox = Aabb::from_boxes(bbox, object.bounding_box());
    }
    bbox.assert_not_infinite();
    bbox
}

impl World {
    pub fn new(objects: Vec<Object>) -> Self {
        Self {
            bounding_box: make_bounding_box(&objects),
            objects,
        }
    }

    #[allow(unused)]
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    pub fn into_objects(self) -> Vec<Object> {
        self.objects
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        let mut record = None;
        let mut closest_so_far = hit_range.max;

        for object in &self.objects {
            let range = Range {
                min: hit_range.min,
                max: closest_so_far,
            };

            if let Some(hit) = object.hit(ray, range) {
                closest_so_far = hit.distance;
                record = Some(hit);
            }
        }

        record
    }

    fn bounding_box(&self) -> Aabb {
        self.bounding_box
    }

    fn id(&self) -> u32 {
        0
    }
}

#[derive(Debug)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    pub material: Arc<Material>,
    bounding_box: Aabb,
    id: u32,
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32, material: Arc<Material>) -> Self {
        let radius_vec = Vec3::new(radius, radius, radius);
        let bounding_box = Aabb::from_points(center - radius_vec, center + radius_vec);
        let id = get_id();

        Sphere {
            center,
            radius,
            material,
            bounding_box,
            id,
        }
    }

    fn get_uv(p: Point3<f32>) -> TextureCoordinates {
        use std::f32::consts::PI;
        // p: a given point on the sphere of radius one, centered at the origin.
        // u: returned value [0,1] of angle around the Y axis from X=-1.
        // v: returned value [0,1] of angle from Y=-1 to Y=+1.
        //     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
        //     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
        //     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>

        let theta = (-p.y).acos();
        let phi = f32::atan2(-p.z, p.x) + PI;

        TextureCoordinates {
            u: phi / (2.0 * PI),
            v: theta / PI,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            None
        } else {
            let sqrtd = discriminant.sqrt();
            let mut root = (h - sqrtd) / a;
            if !hit_range.surrounds(root) {
                root = (h + sqrtd) / a;
                if !hit_range.surrounds(root) {
                    return None;
                }
            }
            let point = ray.evaluate(root);
            let outward_normal = (point - self.center) / self.radius;
            Some(HitRecord::new(
                ray,
                outward_normal,
                point,
                root,
                self.material.clone(),
                Self::get_uv(point),
            ))
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bounding_box
    }

    fn id(&self) -> u32 {
        self.id
    }
}

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
    pub fn new(q: Point3<f32>, v: Vec3<f32>, u: Vec3<f32>, material: Arc<Material>) -> Self {
        let diagonal1 = Aabb::from_points(q, q + u + v);
        let diagonal2 = Aabb::from_points(q + u, q + v);
        let bbox = Aabb::from_boxes(diagonal1, diagonal2);
        let id = get_id();
        let normal = u.cross(v).unit();
        let d = normal.dot(q);
        let w = normal / normal.dot(normal);

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
    if !Range::UNIT.contains(a) || !Range::UNIT.contains(b) {
        None
    } else {
        Some(TextureCoordinates { u: a, v: b })
    }
}
impl Hittable for Quad {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        let denom = self.normal.dot(ray.direction);

        // ray is parallel to the plane
        if denom.abs() < 1e-8 {
            return None;
        }

        let t = self.d - self.normal.dot(ray.origin) / denom;
        if !hit_range.contains(t) {
            return None;
        }

        let intersection = ray.evaluate(t);
        let planar_hit_vector = intersection - self.q;
        let alpha = self.w.dot(planar_hit_vector.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hit_vector));

        is_interior(alpha, beta).map(|tex_coords| HitRecord {
            point: intersection,
            distance: t,
            material: self.material.clone(),
            // TODO
            front_facing: false,
            normal: self.normal,
            tex_coords,
        })
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn id(&self) -> u32 {
        self.id
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
