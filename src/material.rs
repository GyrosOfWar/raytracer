use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::{
    object::HitRecord,
    random::random,
    ray::Ray,
    texture::{HasColorValue, Texture, TextureBuilder, TextureCoordinates},
    vec3::{self, random::gen_unit_vector, reflect, refract, Point3, Vec3},
};

pub struct ScatterResult {
    pub attenuation: Vec3<f32>,
    pub scattered: Ray<f32>,
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
    fn scatter(&self, ray: &Ray<f32>, hit: &HitRecord) -> Option<ScatterResult>;

    fn emit(&self, _: TextureCoordinates, _: Point3<f32>) -> Point3<f32> {
        // default material does not emit anything
        Point3::zero()
    }
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

#[derive(Debug)]
pub struct DiffuseLight {
    pub texture: Arc<Texture>,
}

impl Scatterable for DiffuseLight {
    fn scatter(&self, _: &Ray<f32>, _: &HitRecord) -> Option<ScatterResult> {
        None
    }

    fn emit(&self, uv: TextureCoordinates, point: Point3<f32>) -> Point3<f32> {
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
    Ggx(Ggx),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
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

#[derive(Debug)]
pub struct Mix {
    pub left: Arc<Material>,
    pub right: Arc<Material>,
    pub factor: f32,
}

impl Scatterable for Mix {
    fn scatter(&self, ray: &Ray<f32>, hit: &HitRecord) -> Option<ScatterResult> {
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
pub struct Ggx {
    pub roughness: f32,
}

impl Scatterable for Ggx {
    fn scatter(&self, ray: &Ray<f32>, hit: &HitRecord) -> Option<ScatterResult> {
        todo!()
    }
}

pub mod helpers {
    use std::sync::Arc;

    use crate::{
        texture::{solid, SolidColor, Texture},
        vec3::{Point3, Vec3},
    };

    use super::{Dielectric, DiffuseLight, Lambertian, Material, Metal, Mix};

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

    pub fn diffuse_light(color: Point3<f32>) -> Arc<Material> {
        Arc::new(Material::DiffuseLight(DiffuseLight {
            texture: solid(color),
        }))
    }

    pub fn mix(left: Arc<Material>, right: Arc<Material>, factor: f32) -> Arc<Material> {
        Arc::new(Material::Mix(Mix {
            left,
            right,
            factor,
        }))
    }
}
