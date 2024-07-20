use std::ops::Div;

use glam::Vec2;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::spectrum::{inner_product, DenselySampled, PiecewiseLinear, Spectrum};

pub const CIE_Y_INTEGRAL: f32 = 106.856895;
pub static CIE_XYZ: Lazy<CieXyz> = Lazy::new(|| CieXyz::load());

pub struct CieXyz {
    pub x: Spectrum,
    pub y: Spectrum,
    pub z: Spectrum,
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
            serde_json::from_str(include_str!("../data/cie-xyz.json")).unwrap();
        let x = PiecewiseLinear::new(object.lambda.clone(), object.x);
        let y = PiecewiseLinear::new(object.lambda.clone(), object.y);
        let z = PiecewiseLinear::new(object.lambda, object.z);

        CieXyz {
            x: DenselySampled::from_spectrum(x.into()).into(),
            y: DenselySampled::from_spectrum(y.into()).into(),
            z: DenselySampled::from_spectrum(z.into()).into(),
        }
    }
}

#[derive(Debug)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Xyz {
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

impl From<Spectrum> for Xyz {
    fn from(value: Spectrum) -> Self {
        Xyz {
            x: inner_product(&CIE_XYZ.x, &value),
            y: inner_product(&CIE_XYZ.y, &value),
            z: inner_product(&CIE_XYZ.z, &value),
        }
    }
}

#[derive(Debug)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[cfg(test)]
mod tests {
    use super::Xyz;
    use crate::spectrum::{Constant, Spectrum};

    #[test]
    fn test_xyz_from_spectrum() {
        let spectrum: Spectrum = Constant { c: 400.0 }.into();
        let xyz: Xyz = spectrum.into();
        assert_ne!(xyz.x, 0.0);
        assert_ne!(xyz.y, 0.0);
        assert_ne!(xyz.z, 0.0);
    }
}
