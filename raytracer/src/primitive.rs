use std::sync::Arc;

use enum_dispatch::enum_dispatch;

use crate::bounds::Bounds3;
use crate::ray::RayLike;
use crate::shape::{Object, Shape, ShapeIntersection};
use crate::transform::Transform;

#[enum_dispatch]
pub trait Primitive: Send + Sync {
    fn bounds(&self) -> Bounds3;

    fn intersect(&self, ray: impl RayLike, t_max: f32) -> Option<ShapeIntersection>;

    fn intersect_p(&self, ray: impl RayLike, t_max: f32) -> bool;
}

#[derive(Debug)]
#[enum_dispatch(Primitive)]
pub enum PrimitiveObject {
    Simple(SimplePrimitive),
    Transformed(TransformedPrimitive),
}

#[derive(Debug)]
pub struct SimplePrimitive {
    shape: Object,
    // material: Material
}

impl Primitive for SimplePrimitive {
    fn bounds(&self) -> Bounds3 {
        self.shape.bounds()
    }

    fn intersect(&self, ray: impl RayLike, t_max: f32) -> Option<ShapeIntersection> {
        self.shape.intersect(ray, t_max)
    }

    fn intersect_p(&self, ray: impl RayLike, t_max: f32) -> bool {
        self.shape.intersect_p(ray, t_max)
    }
}

#[derive(Debug)]
pub struct TransformedPrimitive {
    primitive: Box<PrimitiveObject>,
    transform: Arc<Transform>,
}

impl Primitive for TransformedPrimitive {
    fn bounds(&self) -> Bounds3 {
        todo!()
    }

    fn intersect(&self, ray: impl RayLike, t_max: f32) -> Option<ShapeIntersection> {
        todo!()
    }

    fn intersect_p(&self, ray: impl RayLike, t_max: f32) -> bool {
        todo!()
    }
}
