use std::sync::{
    atomic::{AtomicU64, Ordering},
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

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn get_id() -> u64 {
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

    fn id(&self) -> u64;
}

#[enum_dispatch(Hittable)]
#[derive(Debug)]
pub enum Object {
    Sphere(Sphere),
    BvhNode(BvhNode),
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

    fn id(&self) -> u64 {
        0
    }
}

#[derive(Debug)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    pub material: Arc<Material>,
    bounding_box: Aabb,
    id: u64,
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
                // TODO
                TextureCoordinates { u: 0.0, v: 0.0 },
            ))
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bounding_box
    }

    fn id(&self) -> u64 {
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
