use crate::ray::Ray;

#[derive(Debug)]
pub struct Intersection {}

pub trait Integrate {
    fn intersect(&self, ray: &Ray, t_max: f32) -> Option<Intersection>;
}
