use crate::vec3::{Point3, Vec3};

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vec3<f32>,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vec3<f32>) -> Self {
        Ray { origin, direction }
    }

    pub fn evaluate(&self, t: f32) -> Point3<f32> {
        self.origin + (self.direction * t)
    }
}
