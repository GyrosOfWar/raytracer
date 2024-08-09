use crate::{
    bounds::Bounds3,
    math::DirectionCone,
    ray::RayLike,
    vec::{Point2, Point3, Vec3},
};

pub mod triangle_mesh;

#[derive(Debug)]
pub struct ShapeIntersection {
    pub interaction: SurfaceInteraction,
    pub t_hit: f32,
}

#[derive(Debug)]
pub struct ShapeSample {
    pub interaction: SurfaceInteraction,
    pub pdf: f32,
}

// skipping the MediumInteraction because I'm not doing volumetrics
#[derive(Debug)]
pub struct SurfaceInteraction {
    pub point: Point3,
    pub wo: Vec3,
    pub normal: Vec3,
    pub uv: Point2,
}

pub trait Shape {
    /// Returns the bounds of the shape.
    fn bounds(&self) -> Bounds3;

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
