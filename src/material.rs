use std::sync::Arc;

use enum_dispatch::enum_dispatch;

use crate::object::HitRecord;
use crate::random::random;
use crate::ray::Ray;
use crate::texture::{HasColorValue, Texture, TextureCoordinates};
use crate::vec3::random::gen_unit_vector;
use crate::vec3::{self, reflect, refract, Color, Point3, Vec3, Vec3Ext};

pub struct ScatterResult {
    pub attenuation: Color,
    pub scattered: Ray,
}

impl ScatterResult {
    pub fn mix(self, other: ScatterResult, factor: f32) -> ScatterResult {
        ScatterResult {
            attenuation: self.attenuation * factor + other.attenuation * (1.0 - factor),
            scattered: self.scattered,
        }
    }
}

#[enum_dispatch]
pub trait Scatterable {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterResult>;

    fn emit(&self, _: TextureCoordinates, _: Point3<f32>) -> Color {
        // default material does not emit anything
        Vec3::default()
    }
}

#[derive(Debug)]
pub struct Lambertian {
    pub texture: Arc<Texture>,
}

impl Scatterable for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<ScatterResult> {
        let mut scatter_direction = hit.normal + vec3::random::gen_unit_vector();
        if scatter_direction.near_zero() {
            scatter_direction = hit.normal;
        }
        Some(ScatterResult {
            scattered: Ray::new(hit.point, scatter_direction),
            attenuation: self.texture.value_at(hit.tex_coords, hit.point),
        })
    }
}

#[derive(Debug)]
pub struct Metal {
    pub albedo: Color,
    pub fuzz: f32,
}

impl Scatterable for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterResult> {
        let reflected = reflect(ray.direction, hit.normal);
        let reflected = reflected.normalize() + (gen_unit_vector() * self.fuzz);
        Some(ScatterResult {
            scattered: Ray::new(hit.point, reflected),
            attenuation: self.albedo,
        })
    }
}

#[derive(Debug)]
pub struct Dielectric {
    pub refraction_index: f32,
}

impl Scatterable for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterResult> {
        let ri = if hit.front_facing {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray.direction.normalize();
        let cos_theta = (-unit_direction).dot(&hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction = if cannot_refract || reflectance(cos_theta, ri) > random() {
            reflect(unit_direction, hit.normal)
        } else {
            refract(unit_direction, hit.normal, ri)
        };

        Some(ScatterResult {
            attenuation: Vec3::new(1.0, 1.0, 1.0),
            scattered: Ray::new(hit.point, direction),
        })
    }
}

/// Schlick's approximation for reflectance.
fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
    let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    r0 *= r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

#[derive(Debug)]
pub struct DiffuseLight {
    pub texture: Arc<Texture>,
}

impl Scatterable for DiffuseLight {
    fn scatter(&self, _: &Ray, _: &HitRecord) -> Option<ScatterResult> {
        None
    }

    fn emit(&self, uv: TextureCoordinates, point: Point3<f32>) -> Color {
        self.texture.value_at(uv, point)
    }
}

#[enum_dispatch(Scatterable)]
#[derive(Debug)]
pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
    Mix(Mix),
    TrowbridgeReitz(TrowbridgeReitz),
}

#[derive(Debug)]
pub struct Mix {
    pub left: Arc<Material>,
    pub right: Arc<Material>,
    pub factor: f32,
}

impl Scatterable for Mix {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterResult> {
        let left = self.left.scatter(ray, hit);
        let right = self.right.scatter(ray, hit);
        match (left, right) {
            (Some(l), Some(r)) => Some(l.mix(r, self.factor)),
            (None, Some(r)) => Some(r),
            (Some(l), None) => Some(l),
            (None, None) => None,
        }
    }
}

#[derive(Debug)]
pub struct TrowbridgeReitz {
    pub roughness: f32,
}

impl Scatterable for TrowbridgeReitz {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterResult> {
        todo!()
    }
}

pub mod helpers {
    use std::sync::Arc;

    use super::{Lambertian, Material};
    use crate::texture::{SolidColor, Texture};
    use crate::vec3::Vec3;

    pub fn lambertian(albedo: Vec3<f32>) -> Arc<Material> {
        Arc::new(Material::Lambertian(Lambertian {
            texture: Arc::new(Texture::SolidColor(SolidColor { albedo })),
        }))
    }

    pub fn lambertian_texture(texture: Arc<Texture>) -> Arc<Material> {
        Arc::new(Material::Lambertian(Lambertian { texture }))
    }
}
