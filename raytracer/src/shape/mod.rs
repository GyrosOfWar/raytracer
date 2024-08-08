use crate::{aabb::Aabb, ray::RayLike, vec::Point2};

#[derive(Debug)]
pub struct ShapeIntersection {}

pub struct ShapeSample {}

pub trait Shape {
    fn bounds(&self) -> Aabb;

    fn intersect(&self, ray: impl RayLike, t_max: f32) -> Option<ShapeIntersection>;

    fn intersect_p(&self, ray: impl RayLike, t_max: f32) -> bool;

    fn sample(&self, u: Point2) -> ShapeSample;

    fn pdf(&self) -> f32;
}
