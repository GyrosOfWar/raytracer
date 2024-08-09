use crate::vec::{Point3, Vec3};

pub trait RayLike {
    fn evaluate(&self, t: f32) -> Point3;
    fn direction(&self) -> Vec3;
    fn origin(&self) -> Point3;
}

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Ray { origin, direction }
    }

    pub fn with_differentials(self, differential: Differential) -> RayDifferential {
        RayDifferential {
            origin: self.origin,
            direction: self.direction,
            differential,
        }
    }
}

impl RayLike for Ray {
    fn evaluate(&self, t: f32) -> Point3 {
        self.origin + (self.direction * t)
    }

    fn direction(&self) -> Vec3 {
        self.direction
    }

    fn origin(&self) -> Point3 {
        self.origin
    }
}

#[derive(Debug, Clone)]
pub struct RayDifferential {
    pub origin: Point3,
    pub direction: Vec3,
    pub differential: Differential,
}

impl RayLike for RayDifferential {
    fn evaluate(&self, t: f32) -> Point3 {
        self.origin + (self.direction * t)
    }

    fn direction(&self) -> Vec3 {
        self.direction
    }

    fn origin(&self) -> Point3 {
        self.origin
    }
}

impl RayDifferential {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
            differential: Default::default(),
        }
    }

    pub fn scale_differentials(&mut self, s: f32) {
        self.differential.rx_origin = self.origin + (self.differential.rx_origin - self.origin) * s;
        self.differential.ry_origin = self.origin + (self.differential.ry_origin - self.origin) * s;
        self.differential.rx_direction =
            self.direction + (self.differential.rx_direction - self.direction) * s;
        self.differential.ry_direction =
            self.direction + (self.differential.ry_direction - self.direction) * s;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Differential {
    pub rx_origin: Point3,
    pub ry_origin: Point3,
    pub rx_direction: Vec3,
    pub ry_direction: Vec3,
}
