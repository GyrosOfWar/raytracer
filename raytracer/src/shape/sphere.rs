use crate::bounds::Bounds3;
use crate::math::DirectionCone;
use crate::ray::RayLike;
use crate::shape::{Shape, ShapeIntersection, ShapeSample, SurfaceInteraction};
use crate::vec::Point2;

#[derive(Debug, Clone)]
pub struct Sphere {
    radius: f32,
}

impl Shape for Sphere {
    fn bounds(&self) -> Bounds3 {
        todo!()
    }

    fn normal_bounds(&self) -> DirectionCone {
        todo!()
    }

    fn intersect(&self, ray: impl RayLike, t_max: f32) -> Option<ShapeIntersection> {
        todo!()
    }

    fn intersect_p(&self, ray: impl RayLike, t_max: f32) -> bool {
        todo!()
    }

    fn sample(&self, u: Point2) -> ShapeSample {
        todo!()
    }

    fn pdf(&self, interaction: &SurfaceInteraction) -> f32 {
        todo!()
    }

    fn area(&self) -> f32 {
        todo!()
    }
}
