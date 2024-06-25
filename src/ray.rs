use num_traits::Float;

use crate::vec3::{Point3, Vec3};

pub struct Ray<T: Float> {
    pub origin: Point3<T>,
    pub direction: Vec3<T>,
}

impl<T: Float> Ray<T> {
    pub fn new(origin: Point3<T>, direction: Vec3<T>) -> Self {
        Ray { origin, direction }
    }

    pub fn evaluate(&self, t: T) -> Point3<T> {
        self.origin + (self.direction * t)
    }
}
