use std::ops::{Div, Mul};
use std::sync::LazyLock;

use serde::Deserialize;

use crate::spectrum::{inner_product, DenselySampled, HasWavelength, PiecewiseLinear};
use crate::vec::{Point2, Vec3, VectorLike};

pub const CIE_Y_INTEGRAL: f32 = 106.856895;
pub static CIE_XYZ: LazyLock<CieXyz> = LazyLock::new(CieXyz::load);

#[derive(Debug, Clone, Copy)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Xyz> for Vec3 {
    fn from(value: Xyz) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

impl From<Vec3> for Xyz {
    fn from(value: Vec3) -> Self {
        Xyz {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl Xyz {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Xyz { x, y, z }
    }

    pub fn from_xy(xy: Point2) -> Self {
        Self::from_xy_y(xy, 1.0)
    }

    pub fn from_xy_y(xy: Point2, y: f32) -> Self {
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

    pub fn xy(&self) -> Point2 {
        Point2::new(
            self.x / (self.x + self.y + self.z),
            self.y / (self.x + self.y + self.z),
        )
    }
}

impl Mul<f32> for Xyz {
    type Output = Xyz;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
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

impl<'a, T: HasWavelength> From<&'a T> for Xyz {
    fn from(value: &'a T) -> Self {
        let integral = Xyz {
            x: inner_product(&CIE_XYZ.x, value),
            y: inner_product(&CIE_XYZ.y, value),
            z: inner_product(&CIE_XYZ.z, value),
        };

        integral / CIE_Y_INTEGRAL
    }
}

impl VectorLike<3, f32> for Xyz {
    fn component(&self, index: usize) -> f32 {
        assert!(index < 3, "Index out of bounds");
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => unreachable!(),
        }
    }

    fn data(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    fn from_data(data: [f32; 3]) -> Self {
        Xyz {
            x: data[0],
            y: data[1],
            z: data[2],
        }
    }
}

pub struct CieXyz {
    pub x: DenselySampled,
    pub y: DenselySampled,
    pub z: DenselySampled,
}

impl CieXyz {
    fn load() -> Self {
        #[derive(Deserialize)]
        struct CieXyzFile {
            x: Vec<f32>,
            y: Vec<f32>,
            z: Vec<f32>,
            lambda: Vec<f32>,
        }

        let object: CieXyzFile =
            serde_json::from_str(include_str!("../../data/cie-xyz.json")).unwrap();
        let x = PiecewiseLinear::new(object.lambda.clone(), object.x);
        let y = PiecewiseLinear::new(object.lambda.clone(), object.y);
        let z = PiecewiseLinear::new(object.lambda, object.z);

        CieXyz {
            x: DenselySampled::from_spectrum(x),
            y: DenselySampled::from_spectrum(y),
            z: DenselySampled::from_spectrum(z),
        }
    }
}
