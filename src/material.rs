use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::{
    object::HitRecord,
    random::random,
    ray::Ray,
    texture::{HasColorValue, SolidColor, Texture},
    vec3::{self, random::gen_unit_vector, reflect, refract, Vec3},
};

pub struct ScatterResult {
    pub attenuation: Vec3<f32>,
    pub scattered: Ray<f32>,
}

#[enum_dispatch]
pub trait Scatterable {
    fn scatter(&self, ray: &Ray<f32>, hit: &HitRecord) -> Option<ScatterResult>;
}

#[derive(Debug)]
pub struct Lambertian {
    pub texture: Arc<Texture>,
}

impl Scatterable for Lambertian {
    fn scatter(&self, _ray: &Ray<f32>, hit: &HitRecord) -> Option<ScatterResult> {
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
    pub albedo: Vec3<f32>,
    pub fuzz: f32,
}

impl Scatterable for Metal {
    fn scatter(&self, ray: &Ray<f32>, hit: &HitRecord) -> Option<ScatterResult> {
        let reflected = reflect(ray.direction, hit.normal);
        let reflected = reflected.unit() + (gen_unit_vector() * self.fuzz);
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
    fn scatter(&self, ray: &Ray<f32>, hit: &HitRecord) -> Option<ScatterResult> {
        let ri = if hit.front_facing {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray.direction.unit();
        let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
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

#[enum_dispatch(Scatterable)]
#[derive(Debug)]
pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
}

pub fn lambertian(albedo: Vec3<f32>) -> Arc<Material> {
    Arc::new(Material::Lambertian(Lambertian {
        texture: Arc::new(Texture::SolidColor(SolidColor { albedo })),
    }))
}

pub fn lambertian_texture(texture: Arc<Texture>) -> Arc<Material> {
    Arc::new(Material::Lambertian(Lambertian { texture }))
}

pub fn metal(albedo: Vec3<f32>, fuzz: f32) -> Arc<Material> {
    Arc::new(Material::Metal(Metal { albedo, fuzz }))
}

pub fn dielectric(index: f32) -> Arc<Material> {
    Arc::new(Material::Dielectric(Dielectric {
        refraction_index: index,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MaterialBuilder {
    Lambertian(LambertianBuilder),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LambertianBuilder {
    pub texture: TextureBuilder,
}

impl From<MaterialBuilder> for Material {
    fn from(value: MaterialBuilder) -> Self {
        match value {
            MaterialBuilder::Lambertian(l) => Material::Lambertian(Lambertian {
                texture: Arc::new(l.texture.into()),
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TextureBuilder {}

impl From<TextureBuilder> for Texture {
    fn from(value: TextureBuilder) -> Self {
        todo!()
    }
}
