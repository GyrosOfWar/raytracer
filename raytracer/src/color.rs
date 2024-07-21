use std::io::BufRead;
use std::ops::Div;
use std::path::Path;
use std::sync::Arc;

use glam::{Mat3A, Vec2, Vec3A};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::math::{evaluate_polynomial, lerp};
use crate::spectrum::{
    inner_product, DenselySampled, HasWavelength, PiecewiseLinear, Spectrum, LAMBDA_MAX, LAMBDA_MIN,
};
use crate::util::find_interval;
use crate::Result;

pub const CIE_Y_INTEGRAL: f32 = 106.856895;
pub static CIE_XYZ: Lazy<CieXyz> = Lazy::new(CieXyz::load);

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

#[derive(Debug)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<Vec3A> for Rgb {
    fn from(value: Vec3A) -> Self {
        Rgb {
            r: value.x,
            g: value.y,
            b: value.z,
        }
    }
}

impl From<Rgb> for Vec3A {
    fn from(value: Rgb) -> Self {
        Vec3A::new(value.r, value.g, value.b)
    }
}

impl Rgb {
    pub fn max_component_index(&self) -> u8 {
        if self.r > self.g {
            if self.r > self.b {
                0
            } else {
                2
            }
        } else if self.g > self.b {
            1
        } else {
            2
        }
    }

    pub fn component(&self, idx: u8) -> f32 {
        match idx {
            0 => self.r,
            1 => self.g,
            _ => self.b,
        }
    }
}

#[derive(Debug)]
pub struct RgbSigmoidPolynomial {
    pub c0: f32,
    pub c1: f32,
    pub c2: f32,
}

// implements the spectrum interface for convenience,
// but is itself not a spectrum, so left out of the Spectrum enum
impl HasWavelength for RgbSigmoidPolynomial {
    fn evaluate(&self, lambda: f32) -> f32 {
        sigmoid(evaluate_polynomial(&[self.c0, self.c1, self.c2], lambda))
    }

    fn max_value(&self) -> f32 {
        let result = self.evaluate(360.0).max(self.evaluate(830.0));
        let lambda = -self.c1 / (2.0 * self.c0);
        if (LAMBDA_MIN..LAMBDA_MAX).contains(&lambda) {
            result.max(self.evaluate(lambda))
        } else {
            result
        }
    }
}

fn sigmoid(x: f32) -> f32 {
    if x.is_infinite() {
        if x > 0.0 {
            1.0
        } else {
            0.0
        }
    } else {
        0.5 + x / (2.0 * (1.0 + x * x).sqrt())
    }
}

const RES: usize = 64;

#[derive(Serialize, Deserialize)]
pub struct CoefficientsFile {
    pub coefficients: Vec<f32>,
    pub scale: Vec<f32>,
}

impl CoefficientsFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        use std::fs::File;
        use std::io::BufReader;

        let reader = BufReader::new(File::open(path)?);
        let value = bincode::deserialize_from(reader)?;
        Ok(value)
    }
}

pub struct RgbToSpectrumTable {
    z_nodes: Box<[f32]>,
    coefficients: Box<[f32]>,
}

impl RgbToSpectrumTable {
    pub fn new(file: CoefficientsFile) -> Self {
        RgbToSpectrumTable {
            coefficients: file.coefficients.into_boxed_slice(),
            z_nodes: file.scale.into_boxed_slice(),
        }
    }

    fn coeff(&self, i1: usize, i2: usize, i3: usize, i4: usize, i5: usize) -> f32 {
        const DIM_2: usize = RES;
        const DIM_3: usize = RES;
        const DIM_4: usize = RES;
        const DIM_5: usize = 3;

        let index = i1 * DIM_2 * DIM_3 * DIM_4 * DIM_5
            + i2 * DIM_3 * DIM_4 * DIM_5
            + i3 * DIM_4 * DIM_5
            + i4 * DIM_5
            + i5;

        self.coefficients[index]
    }

    pub fn evaluate(&self, rgb: Rgb) -> RgbSigmoidPolynomial {
        if rgb.r == rgb.g && rgb.g == rgb.b {
            RgbSigmoidPolynomial {
                c0: 0.0,
                c1: 0.0,
                c2: (rgb.r - 0.5) / (rgb.r * (1.0 - rgb.r)).sqrt(),
            }
        } else {
            let max_c = rgb.max_component_index();
            let z = rgb.component(max_c);
            let x = (rgb.component((max_c + 1) % 3) * (RES as f32 - 1.0)) / z;
            let y = (rgb.component((max_c + 2) % 3) * (RES as f32 - 1.0)) / z;
            let xi = (x as usize).min(RES - 2);
            let yi = (y as usize).min(RES - 2);
            let zi = find_interval(&self.z_nodes, z);
            let dx = x - xi as f32;
            let dy = y - yi as f32;
            let dz = (z - self.z_nodes[zi]) / (self.z_nodes[zi + 1] - self.z_nodes[zi]);

            let mut c = [0.0f32; 3];
            for (i, value) in c.iter_mut().enumerate() {
                let co = |dx: usize, dy: usize, dz: usize| {
                    self.coeff(max_c as usize, zi + dz, yi + dy, xi + dx, i)
                };
                *value = lerp(
                    dz,
                    lerp(
                        dy,
                        lerp(dx, co(0, 0, 0), co(1, 0, 0)),
                        lerp(dx, co(0, 1, 0), co(1, 1, 0)),
                    ),
                    lerp(
                        dy,
                        lerp(dx, co(0, 0, 1), co(1, 0, 1)),
                        lerp(dx, co(0, 1, 1), co(1, 1, 1)),
                    ),
                );
            }
            RgbSigmoidPolynomial {
                c0: c[0],
                c1: c[1],
                c2: c[2],
            }
        }
    }
}

pub struct RgbColorSpace {
    pub r: Vec2,
    pub g: Vec2,
    pub b: Vec2,
    pub w: Vec2,
    pub illuminant: Spectrum,
    spectrum_table: Arc<RgbToSpectrumTable>,
    pub rgb_from_xyz: Mat3A,
    pub xyz_from_rgb: Mat3A,
}

impl RgbColorSpace {
    pub fn new(
        r: Vec2,
        g: Vec2,
        b: Vec2,
        illuminant: Spectrum,
        spectrum_table: Arc<RgbToSpectrumTable>,
    ) -> Self {
        let w_xyz = Xyz::from(&illuminant);
        let w = w_xyz.xy();
        let xyz_r = Xyz::from_xy(r);
        let xyz_g = Xyz::from_xy(g);
        let xyz_b = Xyz::from_xy(b);
        let rgb = Mat3A::from_cols(
            Vec3A::new(xyz_r.x, xyz_g.x, xyz_b.x),
            Vec3A::new(xyz_r.y, xyz_g.y, xyz_b.y),
            Vec3A::new(xyz_r.z, xyz_g.z, xyz_b.z),
        );
        let c = rgb.inverse() * Vec3A::from(w_xyz);
        let xyz_from_rgb = rgb
            * Mat3A::from_cols(
                Vec3A::new(c[0], 0.0, 0.0),
                Vec3A::new(0.0, c[1], 0.0),
                Vec3A::new(0.0, 0.0, c[2]),
            );
        let rgb_from_xyz = xyz_from_rgb.inverse();

        Self {
            r,
            g,
            b,
            w,
            illuminant,
            spectrum_table,
            rgb_from_xyz,
            xyz_from_rgb,
        }
    }

    pub fn to_rgb(&self, xyz: Xyz) -> Rgb {
        let vec = self.rgb_from_xyz * Vec3A::from(xyz);
        vec.into()
    }

    pub fn to_xyz(&self, rgb: Rgb) -> Xyz {
        let vec = self.xyz_from_rgb * Vec3A::from(rgb);
        vec.into()
    }

    pub fn convert_color_space(from: &RgbColorSpace, to: &RgbColorSpace) -> Mat3A {
        to.rgb_from_xyz * from.xyz_from_rgb
    }
}

#[cfg(test)]
mod tests {
    use super::{CoefficientsFile, Rgb, RgbToSpectrumTable};
    use crate::spectrum::HasWavelength;
    use color_eyre::Result;

    #[test]
    fn test_rgb_component() {
        let rgb = Rgb {
            r: 1.0,
            g: 0.5,
            b: 0.7,
        };
        assert_eq!(rgb.max_component_index(), 0);
        let rgb = Rgb {
            r: 0.5,
            g: 1.0,
            b: 0.2,
        };
        assert_eq!(rgb.max_component_index(), 1);
        let rgb = Rgb {
            r: 0.2,
            g: 0.7,
            b: 1.0,
        };
        assert_eq!(rgb.max_component_index(), 2);
    }

    #[test]
    fn test_load_spectrum_file() -> Result<()> {
        let paths = &[
            "./data/color-spaces/aces.bin",
            "./data/color-spaces/dci_p3.bin",
            "./data/color-spaces/rec2020.bin",
            "./data/color-spaces/srgb.bin",
        ];

        for path in paths {
            let file = CoefficientsFile::load(path)?;
            assert_eq!(file.coefficients.len(), 3 * 64 * 64 * 64 * 3);
            assert_eq!(file.scale.len(), 64);
        }
        Ok(())
    }

    #[test]
    fn test_create_spectrum_table() -> Result<()> {
        let file = CoefficientsFile::load("./data/color-spaces/srgb.bin")?;
        let table = RgbToSpectrumTable::new(file);
        let sigmoid = table.evaluate(Rgb {
            r: 0.0,
            g: 1.0,
            b: 0.0,
        });
        // sanity check, the entire visible range should be defined
        for lambda in 360..830 {
            let result = sigmoid.evaluate(lambda as f32);
            assert!(result > 0.0);
        }

        Ok(())
    }

    #[test]
    fn test_evaluate_table() -> Result<()> {
        let file = CoefficientsFile::load("./data/color-spaces/srgb.bin")?;
        let table = RgbToSpectrumTable::new(file);

        for r in 0..100 {
            let r = r as f32 / 100.0;
            for g in 0..100 {
                let g = g as f32 / 100.0;
                for b in 0..100 {
                    let b = b as f32 / 100.0;
                    let color = Rgb { r, g, b };
                    let sigmoid = table.evaluate(color);

                    for lambda in 360..830 {
                        let result = sigmoid.evaluate(lambda as f32);
                        assert!(result >= 0.0);
                    }
                }
            }
        }

        Ok(())
    }
}
