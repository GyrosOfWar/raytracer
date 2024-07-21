use std::path::Path;
use std::sync::Arc;

use glam::{Mat3A, Vec2, Vec3A};
use serde::{Deserialize, Serialize};

use super::rgb::{Rgb, RgbSigmoidPolynomial};
use super::xyz::Xyz;
use crate::math::lerp;
use crate::spectrum::Spectrum;
use crate::util::find_interval;
use crate::Result;

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

#[derive(Debug)]
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

#[derive(Debug)]
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
    use super::RgbToSpectrumTable;
    use crate::color::colorspace::CoefficientsFile;
    use crate::color::rgb::Rgb;
    use crate::spectrum::HasWavelength;
    use crate::Result;

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
