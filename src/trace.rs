use num_traits::One;

use crate::{
    ppm::Color,
    ray::Ray,
    vec3::{Point3, Vec3},
};

pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl Range {
    pub fn contains(&self, value: f32) -> bool {
        value >= self.min && value <= self.max
    }
}

pub struct HitRecord {
    pub point: Point3<f32>,
    pub normal: Vec3<f32>,
    pub distance: f32,
    pub front_facing: bool,
}

impl HitRecord {
    pub fn new(
        ray: &Ray<f32>,
        outward_normal: Vec3<f32>,
        point: Point3<f32>,
        distance: f32,
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
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray<f32>, outward_normal: Vec3<f32>) {
        self.front_facing = ray.direction.dot(outward_normal) < 0.0;
        self.normal = if self.front_facing {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord>;
}

#[derive(Debug)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
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
            if !hit_range.contains(root) {
                root = (h + sqrtd) / a;
                if !hit_range.contains(root) {
                    return None;
                }
            }
            let point = ray.evaluate(root);
            let outward_normal = (point - self.center) / self.radius;
            Some(HitRecord::new(ray, outward_normal, point, root))
        }
    }
}

pub fn ray_color(ray: &Ray<f32>) -> Color {
    let center = Point3::new(0.0, 0.0, -1.0);
    let radius = 0.5;
    let sphere = Sphere { center, radius };
    // let intersection = sphere.hit(ray, hit_range)
    let t = 0.0;

    if t >= 0.0 {
        let n = (ray.evaluate(t) - Vec3::new(0.0, 0.0, -1.0)).unit();
        ((n + 1.0) * 0.5).into()
    } else {
        let direction = ray.direction.unit();
        let t = 0.5 * (direction.y + 1.0);
        Vec3::one().lerp(Vec3::new(0.5, 0.7, 1.0), t).into()
    }
}
