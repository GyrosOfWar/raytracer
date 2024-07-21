use std::ops::Div;

use glam::{Vec2, Vec3A};

use super::CIE_XYZ;
use crate::spectrum::{inner_product, Spectrum};

#[derive(Debug)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Xyz> for Vec3A {
    fn from(value: Xyz) -> Self {
        Vec3A::new(value.x, value.y, value.z)
    }
}

impl From<Vec3A> for Xyz {
    fn from(value: Vec3A) -> Self {
        Xyz {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl Xyz {
    pub fn from_xy(xy: Vec2) -> Self {
        Self::from_xy_y(xy, 1.0)
    }

    pub fn from_xy_y(xy: Vec2, y: f32) -> Self {
        if xy.y == 0.0 {
            Xyz {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            Xyz {
                x: xy.x * y / xy.y,
                y,
                z: (1.0 - xy.x - xy.y) * y / xy.y,
            }
        }
    }

    pub fn xy(&self) -> Vec2 {
        Vec2::new(
            self.x / (self.x + self.y + self.z),
            self.y / (self.x + self.y + self.z),
        )
    }
}

impl Div<f32> for Xyz {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Xyz {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<'a> From<&'a Spectrum> for Xyz {
    fn from(value: &'a Spectrum) -> Self {
        Xyz {
            x: inner_product(&CIE_XYZ.x, &value),
            y: inner_product(&CIE_XYZ.y, &value),
            z: inner_product(&CIE_XYZ.z, &value),
        }
    }
}
