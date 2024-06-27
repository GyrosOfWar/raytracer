use std::sync::Arc;

use enum_dispatch::enum_dispatch;

use crate::{
    helpers::random,
    ray::Ray,
    trace::HitRecord,
    vec3::{self, random::gen_unit_vector, reflect, refract, Vec3},
};

#[enum_dispatch]
pub trait Scatterable {
    fn scatter(
        &self,
        ray: &Ray<f32>,
        hit: &HitRecord,
        attenuation: &mut Vec3<f32>,
        scattered: &mut Ray<f32>,
    ) -> bool;
}

#[derive(Debug)]
pub struct Lambertian {
    pub albedo: Vec3<f32>,
}

impl Scatterable for Lambertian {
    fn scatter(
        &self,
        _ray: &Ray<f32>,
        hit: &HitRecord,
        attenuation: &mut Vec3<f32>,
        scattered: &mut Ray<f32>,
    ) -> bool {
        let mut scatter_direction = hit.normal + vec3::random::gen_unit_vector();
        if scatter_direction.near_zero() {
            scatter_direction = hit.normal;
        }
        *scattered = Ray::new(hit.point, scatter_direction);
        *attenuation = self.albedo;
        return true;
    }
}

#[derive(Debug)]
pub struct Metal {
    pub albedo: Vec3<f32>,
    pub fuzz: f32,
}

impl Scatterable for Metal {
    fn scatter(
        &self,
        ray: &Ray<f32>,
        hit: &HitRecord,
        attenuation: &mut Vec3<f32>,
        scattered: &mut Ray<f32>,
    ) -> bool {
        let reflected = reflect(ray.direction, hit.normal);
        let reflected = reflected.unit() + (gen_unit_vector() * self.fuzz);
        *scattered = Ray::new(hit.point, reflected);
        *attenuation = self.albedo;
        return true;
    }
}

#[derive(Debug)]
pub struct Dielectric {
    pub refraction_index: f32,
}

impl Scatterable for Dielectric {
    fn scatter(
        &self,
        ray: &Ray<f32>,
        hit: &HitRecord,
        attenuation: &mut Vec3<f32>,
        scattered: &mut Ray<f32>,
    ) -> bool {
        *attenuation = Vec3::new(1.0, 1.0, 1.0);
        let ri = if hit.front_facing {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray.direction.unit();
        let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction;

        if cannot_refract || reflectance(cos_theta, ri) > random() {
            direction = reflect(unit_direction, hit.normal);
        } else {
            direction = refract(unit_direction, hit.normal, ri);
        }
        *scattered = Ray::new(hit.point, direction);
        return true;
    }
}

fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
    // Use Schlick's approximation for reflectance.
    let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * (1.0 - cosine).powi(5);
}

#[enum_dispatch(Scatterable)]
#[derive(Debug)]
pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
}

pub fn lambertian(albedo: Vec3<f32>) -> Arc<Material> {
    Arc::new(Material::Lambertian(Lambertian { albedo }))
}

pub fn metal(albedo: Vec3<f32>, fuzz: f32) -> Arc<Material> {
    Arc::new(Material::Metal(Metal { albedo, fuzz }))
}

pub fn dielectric(index: f32) -> Arc<Material> {
    Arc::new(Material::Dielectric(Dielectric {
        refraction_index: index,
    }))
}
