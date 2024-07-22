use crate::vec::{Point3, Vec3};

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Ray { origin, direction }
    }

    pub fn evaluate(&self, t: f32) -> Point3 {
        self.origin + (self.direction * t)
    }
}

#[derive(Debug)]
pub struct RayDifferential {}
