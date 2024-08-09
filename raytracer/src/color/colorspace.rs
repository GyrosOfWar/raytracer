use std::path::Path;
use std::sync::{Arc, LazyLock};

use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::rgb::{Rgb, RgbSigmoidPolynomial};
use super::xyz::Xyz;
use crate::math::lerp;
use crate::spectrum::{Spectrum, ILLUM_ACES_D60, STD_ILLUM_D_65};
use crate::vec::{Mat3, Point2, Vec3};
use crate::{util, Result};

const RES: usize = 64;

pub static S_RGB: LazyLock<Arc<RgbColorSpace>> = LazyLock::new(|| {
    let table = RgbToSpectrumTable::new(
        CoefficientsFile::load("../data/color-spaces/srgb.json")
            .expect("failed to load srgb table"),
    );
    Arc::new(RgbColorSpace::new(
        Point2::new(0.64, 0.33),
        Point2::new(0.3, 0.6),
        Point2::new(0.15, 0.06),
        STD_ILLUM_D_65.clone(),
        table,
    ))
});

pub static DCI_P3: LazyLock<Arc<RgbColorSpace>> = LazyLock::new(|| {
    let table = RgbToSpectrumTable::new(
        CoefficientsFile::load("../data/color-spaces/dci_p3.json")
            .expect("failed to load dci_p3 table"),
    );
    Arc::new(RgbColorSpace::new(
        Point2::new(0.68, 0.32),
        Point2::new(0.265, 0.690),
        Point2::new(0.15, 0.06),
        STD_ILLUM_D_65.clone(),
        table,
    ))
});

pub static REC_2020: LazyLock<Arc<RgbColorSpace>> = LazyLock::new(|| {
    let table = RgbToSpectrumTable::new(
        CoefficientsFile::load("../data/color-spaces/rec2020.json")
            .expect("failed to load rec2020 table"),
    );

    Arc::new(RgbColorSpace::new(
        Point2::new(0.708, 0.292),
        Point2::new(0.170, 0.797),
        Point2::new(0.131, 0.046),
        STD_ILLUM_D_65.clone(),
        table,
    ))
});

pub static ACES2065_1: LazyLock<Arc<RgbColorSpace>> = LazyLock::new(|| {
    let table = RgbToSpectrumTable::new(
        CoefficientsFile::load("../data/color-spaces/aces.json")
            .expect("failed to load aces2065-1 table"),
    );
    Arc::new(RgbColorSpace::new(
        Point2::new(0.7347, 0.2653),
        Point2::new(0.0, 1.0),
        Point2::new(0.0001, -0.077),
        ILLUM_ACES_D60.clone(),
        table,
    ))
});

#[derive(Serialize, Deserialize)]
pub struct CoefficientsFile {
    pub resolution: usize,
    pub scale: Vec<f32>,
    pub data: Vec<f32>,
}

impl CoefficientsFile {
    #[instrument(skip(path), name = "CoefficientsFile::load")]
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        use std::fs::File;
        use std::io::BufReader;

        let reader = BufReader::new(File::open(path)?);
        serde_json::from_reader(reader).map_err(From::from)
    }
}

#[derive(Debug)]
pub struct RgbToSpectrumTable {
    z_nodes: Box<[f32]>,
    coefficients: Vec<f32>,
}

impl RgbToSpectrumTable {
    pub fn new(file: CoefficientsFile) -> Self {
        RgbToSpectrumTable {
            coefficients: file.data,
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
                c2: (rgb.r - 0.5) / f32::sqrt(rgb.r * (1.0 - rgb.r)),
            }
        } else {
            let max_c = rgb.max_component_index();
            let z = rgb.component(max_c);
            let x_index = (max_c + 1) % 3;
            let y_index = (max_c + 2) % 3;
            let x = (rgb.component(x_index) * (RES as f32 - 1.0)) / z;
            let y = (rgb.component(y_index) * (RES as f32 - 1.0)) / z;
            let xi = (x as usize).min(RES - 2);
            let yi = (y as usize).min(RES - 2);
            let zi = util::find_interval(self.z_nodes.len(), |idx| self.z_nodes[idx] < z);
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
    pub r: Point2,
    pub g: Point2,
    pub b: Point2,
    pub w: Point2,
    pub illuminant: Arc<Spectrum>,
    pub rgb_from_xyz: Mat3,
    pub xyz_from_rgb: Mat3,
    spectrum_table: RgbToSpectrumTable,
}

impl RgbColorSpace {
    pub fn new(
        r: Point2,
        g: Point2,
        b: Point2,
        illuminant: Arc<Spectrum>,
        spectrum_table: RgbToSpectrumTable,
    ) -> Self {
        let w_xyz = Xyz::from(illuminant.as_ref());
        let w = w_xyz.xy();
        let xyz_r = Xyz::from_xy(r);
        let xyz_g = Xyz::from_xy(g);
        let xyz_b = Xyz::from_xy(b);
        let rgb = Mat3::from_cols(Vec3::from(xyz_r), Vec3::from(xyz_g), Vec3::from(xyz_b));
        let rgb_inv = rgb.inverse();
        let c = rgb_inv * Vec3::from(w_xyz);
        let c_mat = Mat3::from_diagonal(c);

        let xyz_from_rgb = rgb * c_mat;
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
        let vec = self.rgb_from_xyz * Vec3::from(xyz);
        vec.into()
    }

    pub fn to_xyz(&self, rgb: Rgb) -> Xyz {
        let vec = self.xyz_from_rgb * Vec3::from(rgb);
        vec.into()
    }

    pub fn to_rgb_coefficients(&self, rgb: Rgb) -> RgbSigmoidPolynomial {
        self.spectrum_table.evaluate(rgb.clamp_zero())
    }

    pub fn convert_color_space(from: &RgbColorSpace, to: &RgbColorSpace) -> Mat3 {
        to.rgb_from_xyz * from.xyz_from_rgb
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::{RgbToSpectrumTable, ACES2065_1, REC_2020, S_RGB};
    use crate::color::colorspace::CoefficientsFile;
    use crate::color::rgb::Rgb;
    use crate::color::xyz::Xyz;
    use crate::random::random;
    use crate::spectrum::{DenselySampled, HasWavelength, RgbAlbedo, RgbIlluminant};
    use crate::{assert_approx_eq, Result};

    fn for_each_color(func: impl Fn(f32, f32, f32)) {
        for r in 0..50 {
            let r = r as f32 / 100.0;
            for g in 0..50 {
                let g = g as f32 / 100.0;
                for b in 0..50 {
                    let b = b as f32 / 100.0;
                    func(r, g, b)
                }
            }
        }
    }

    #[test]
    #[traced_test]
    fn test_load_spectrum_file() -> Result<()> {
        let paths = &[
            "../data/color-spaces/aces.json",
            "../data/color-spaces/dci_p3.json",
            "../data/color-spaces/rec2020.json",
            "../data/color-spaces/srgb.json",
        ];

        for path in paths {
            let file = CoefficientsFile::load(path)?;
            assert_eq!(file.data.len(), 3 * 64 * 64 * 64 * 3);
            assert_eq!(file.scale.len(), 64);
        }
        Ok(())
    }

    #[test]
    fn rgb_color_space_rgbxyz() {
        let color_spaces = vec![&ACES2065_1, &REC_2020, &S_RGB];

        for cs in color_spaces {
            let xyz = cs.to_xyz(Rgb::new(1.0, 1.0, 1.0));
            let rgb = cs.to_rgb(xyz);

            let eps = 1e-4;

            assert_approx_eq!(1.0, rgb.r, eps);
            assert_approx_eq!(1.0, rgb.g, eps);
            assert_approx_eq!(1.0, rgb.b, eps);
        }
    }

    #[test]
    fn srgb_color_space() {
        let srgb = &S_RGB;

        // Make sure the matrix values are sensible by throwing the x, y, and z
        // basis vectors at it to pull out columns.
        let rgb = srgb.to_rgb(Xyz::new(1.0, 0.0, 0.0));
        let eps = 1e-3;
        assert_approx_eq!(3.2406, rgb.r, eps);
        assert_approx_eq!(-0.9692676, rgb.g, eps);
        assert_approx_eq!(0.0557, rgb.b, eps);

        let rgb = srgb.to_rgb(Xyz::new(0.0, 1.0, 0.0));
        assert_approx_eq!(-1.5372, rgb.r, eps);
        assert_approx_eq!(1.8758, rgb.g, eps);
        assert_approx_eq!(-0.2040, rgb.b, eps);

        let rgb = srgb.to_rgb(Xyz::new(0.0, 0.0, 1.0));
        assert_approx_eq!(-0.4986, rgb.r, eps);
        assert_approx_eq!(0.0415, rgb.g, eps);
        assert_approx_eq!(1.0570, rgb.b, eps);
    }

    #[test]
    fn test_create_spectrum_table() -> Result<()> {
        let file = CoefficientsFile::load("../data/color-spaces/srgb.json")?;
        let table = RgbToSpectrumTable::new(file);
        let sigmoid = table.evaluate(Rgb::new(0.0, 1.0, 0.0));

        // sanity check, the entire visible range should be defined
        for lambda in 360..830 {
            let result = sigmoid.evaluate(lambda as f32);
            assert!(result > 0.0);
        }

        Ok(())
    }

    #[test]
    fn test_evaluate_table() -> Result<()> {
        let file = CoefficientsFile::load("../data/color-spaces/srgb.json")?;
        let table = RgbToSpectrumTable::new(file);

        for_each_color(|r, g, b| {
            let color = Rgb { r, g, b };
            let sigmoid = table.evaluate(color);

            for lambda in 360..830 {
                let result = sigmoid.evaluate(lambda as f32);
                assert!(result >= 0.0);
            }
        });

        Ok(())
    }

    #[test]
    fn round_trip_conversion_srgb() {
        let color_space = &S_RGB;

        for _ in 0..100 {
            let r = random();
            let g = random();
            let b = random();
            let rgb = Rgb::new(r, g, b);
            let spectrum = RgbAlbedo::with_color_space(color_space, rgb);

            let spectrum = DenselySampled::from_fn(|l| {
                let rgb = spectrum.evaluate(l);
                let w = color_space.illuminant.evaluate(l);

                rgb * w
            });

            let xyz = Xyz::from(&spectrum);
            let rgb2 = color_space.to_rgb(xyz);

            let eps = 0.01;
            assert_approx_eq!(rgb.r, rgb2.r, eps);
            assert_approx_eq!(rgb.g, rgb2.g, eps);
            assert_approx_eq!(rgb.b, rgb2.b, eps);
        }
    }

    #[test]
    fn round_trip_conversion_rec_2020() {
        let color_space = &REC_2020;

        for _ in 0..100 {
            let r = 0.1 + 0.7 * random();
            let g = 0.1 + 0.7 * random();
            let b = 0.1 + 0.7 * random();
            let rgb = Rgb::new(r, g, b);
            let spectrum = RgbAlbedo::with_color_space(color_space, rgb);

            let spectrum = DenselySampled::from_fn(|lambda| {
                let rgb = spectrum.evaluate(lambda);
                let w = color_space.illuminant.evaluate(lambda);

                rgb * w
            });

            let xyz = Xyz::from(&spectrum);
            let rgb2 = color_space.to_rgb(xyz);

            let eps = 0.01;
            assert_approx_eq!(rgb.r, rgb2.r, eps);
            assert_approx_eq!(rgb.g, rgb2.g, eps);
            assert_approx_eq!(rgb.b, rgb2.b, eps);
        }
    }

    #[test]
    fn round_trip_conversion_aces() {
        let color_space = &ACES2065_1;

        for _ in 0..100 {
            let r = 0.3 + 0.4 * random();
            let g = 0.3 + 0.4 * random();
            let b = 0.3 + 0.4 * random();
            let rgb = Rgb::new(r, g, b);
            let spectrum = RgbAlbedo::with_color_space(color_space, rgb);

            let spectrum = DenselySampled::from_fn(|lambda| {
                let rgb = spectrum.evaluate(lambda);
                let w = color_space.illuminant.evaluate(lambda);

                rgb * w
            });

            let xyz = Xyz::from(&spectrum);
            let rgb2 = color_space.to_rgb(xyz);

            let eps = 0.01;
            assert_approx_eq!(rgb.r, rgb2.r, eps);
            assert_approx_eq!(rgb.g, rgb2.g, eps);
            assert_approx_eq!(rgb.b, rgb2.b, eps);
        }
    }

    #[test]
    fn illuminant_round_trip_conversion_srgb() {
        let color_space = &S_RGB;

        for _ in 0..100 {
            let r = random();
            let g = random();
            let b = random();
            let rgb = Rgb::new(r, g, b);
            let spectrum = RgbIlluminant::new(color_space, rgb);
            let spectrum = DenselySampled::from_fn(|lambda| spectrum.evaluate(lambda));

            let xyz = Xyz::from(&spectrum);
            let rgb2 = color_space.to_rgb(xyz);

            let eps = 0.01;
            assert_approx_eq!(rgb.r, rgb2.r, eps);
            assert_approx_eq!(rgb.g, rgb2.g, eps);
            assert_approx_eq!(rgb.b, rgb2.b, eps);
        }
    }

    #[test]
    fn illuminant_round_trip_conversion_rec_2020() {
        let color_space = &REC_2020;

        for _ in 0..100 {
            let r = 0.1 + 0.7 * random();
            let g = 0.1 + 0.7 * random();
            let b = 0.1 + 0.7 * random();
            let rgb = Rgb::new(r, g, b);
            let spectrum = RgbIlluminant::new(color_space, rgb);
            let spectrum = DenselySampled::from_fn(|lambda| spectrum.evaluate(lambda));

            let xyz = Xyz::from(&spectrum);
            let rgb2 = color_space.to_rgb(xyz);

            let eps = 0.01;
            assert_approx_eq!(rgb.r, rgb2.r, eps);
            assert_approx_eq!(rgb.g, rgb2.g, eps);
            assert_approx_eq!(rgb.b, rgb2.b, eps);
        }
    }

    #[test]
    fn illuminant_round_trip_conversion_aces() {
        let color_space = &ACES2065_1;

        for _ in 0..100 {
            let r = 0.3 + 0.4 * random();
            let g = 0.3 + 0.4 * random();
            let b = 0.3 + 0.4 * random();
            let rgb = Rgb::new(r, g, b);
            let spectrum = RgbIlluminant::new(color_space, rgb);
            let spectrum = DenselySampled::from_fn(|lambda| spectrum.evaluate(lambda));

            let xyz = Xyz::from(&spectrum);
            let rgb2 = color_space.to_rgb(xyz);

            let eps = 0.01;
            assert_approx_eq!(rgb.r, rgb2.r, eps);
            assert_approx_eq!(rgb.g, rgb2.g, eps);
            assert_approx_eq!(rgb.b, rgb2.b, eps);
        }
    }
}
