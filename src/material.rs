use std::rc::Rc;

use enum_dispatch::enum_dispatch;

use crate::{
    ray::Ray,
    trace::HitRecord,
    vec3::{self, Vec3},
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
}

impl Scatterable for Metal {
    fn scatter(
        &self,
        ray: &Ray<f32>,
        hit: &HitRecord,
        attenuation: &mut Vec3<f32>,
        scattered: &mut Ray<f32>,
    ) -> bool {
        let reflected = ray.direction.reflect(hit.normal);
        *scattered = Ray::new(hit.point, reflected);
        *attenuation = self.albedo;
        return true;
    }
}

#[enum_dispatch(Scatterable)]
#[derive(Debug)]
pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
}

pub fn lambertian(albedo: Vec3<f32>) -> Rc<Material> {
    Rc::new(Material::Lambertian(Lambertian { albedo }))
}

pub fn metal(albedo: Vec3<f32>) -> Rc<Material> {
    Rc::new(Material::Metal(Metal { albedo }))
}
