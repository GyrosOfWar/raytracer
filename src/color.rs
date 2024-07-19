use super::spectrum::{inner_product, DenselySampled, PiecewiseLinear, Spectrum};
use once_cell::sync::Lazy;
use serde::Deserialize;

static CIE_XYZ: Lazy<CieXyz> = Lazy::new(|| CieXyz::load());

#[derive(Deserialize)]
struct CieXyzFile {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    lambda: Vec<f32>,
}
struct CieXyz {
    x: Spectrum,
    y: Spectrum,
    z: Spectrum,
}

impl CieXyz {
    fn load() -> Self {
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

impl From<Spectrum> for Xyz {
    fn from(value: Spectrum) -> Self {
        Xyz {
            x: inner_product(&CIE_XYZ.x, &value),
            y: inner_product(&CIE_XYZ.y, &value),
            z: inner_product(&CIE_XYZ.z, &value),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::spectrum::{Constant, Spectrum};

    use super::Xyz;

    #[test]
    fn test_xyz_from_spectrum() {
        // TODO look at the pbrt test suite
        let spectrum: Spectrum = Constant { c: 400.0 }.into();
        let xyz: Xyz = spectrum.into();
        assert_ne!(xyz.x, 0.0);
        assert_ne!(xyz.y, 0.0);
        assert_ne!(xyz.z, 0.0);
    }
}
