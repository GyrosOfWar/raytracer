use crate::vec::{Point3, Vec3};

#[derive(Debug, Clone)]
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

    pub fn is_nan(&self) -> bool {
        self.origin.is_nan() || self.direction.is_nan()
    }

    pub fn with_differentials(self, differential: Differential) -> RayDifferential {
        RayDifferential {
            origin: self.origin,
            direction: self.direction,
            differential,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RayDifferential {
    pub origin: Point3,
    pub direction: Point3,
    pub differential: Differential,
}

impl RayDifferential {
    pub fn is_nan(&self) -> bool {
        [
            self.differential.rx_origin,
            self.differential.ry_origin,
            self.differential.rx_direction,
            self.differential.ry_direction,
        ]
        .iter()
        .any(|v| v.is_nan())
    }
}

#[derive(Debug, Clone)]
pub struct Differential {
    pub rx_origin: Point3,
    pub ry_origin: Point3,
    pub rx_direction: Vec3,
    pub ry_direction: Vec3,
}
