use crate::{aabb::Aabb, math::DirectionCone, ray::RayLike, vec::Point2};

#[derive(Debug)]
pub struct ShapeIntersection {}

#[derive(Debug)]
pub struct ShapeSample {}

// skipping the MediumInteraction because I'm not doing volumetrics
#[derive(Debug)]
pub struct SurfaceInteraction {}

pub trait Shape {
    /// Returns the bounds of the shape.
    fn bounds(&self) -> Aabb;

    /// Return the range of the surface normal
    fn normal_bounds(&self) -> DirectionCone;

    /// Intersect the ray with the shape and return the intersection.
    fn intersect(&self, ray: impl RayLike, t_max: f32) -> Option<ShapeIntersection>;

    /// Intersect the ray with the shape and return whether there is an intersection.
    fn intersect_p(&self, ray: impl RayLike, t_max: f32) -> bool;

    /// Sample a point on the shape.
    fn sample(&self, u: Point2) -> ShapeSample;

    /// Compute the PDF of the shape.
    fn pdf(&self) -> f32;

    /// Get the surface are of the shape
    fn area(&self) -> f32;
}
