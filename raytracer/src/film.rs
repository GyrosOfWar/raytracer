use std::sync::Arc;

use glam::{vec3, Mat3A, U64Vec2, Vec2, Vec3A};
use once_cell::sync::Lazy;

use crate::camera::Bounds2i;
use crate::color::colorspace::{RgbColorSpace, S_RGB};
use crate::color::rgb::Rgb;
use crate::color::xyz::{Xyz, CIE_XYZ};
use crate::filter::{Filter, ReconstructionFilter};
use crate::math::linear_least_squares;
use crate::spectrum::{
    inner_product, HasWavelength, NamedSpectra, PiecewiseLinear, SampledSpectrum,
    SampledWavelengths, Spectrum, LAMBDA_MAX, LAMBDA_MIN,
};

static SWATCH_REFLECTANCES: Lazy<Vec<Spectrum>> = Lazy::new(load_swatch_reflectances);

#[derive(Debug)]
pub struct FilmBaseParameters {
    full_resolution: U64Vec2,
    pixel_bounds: Bounds2i,
    filter: ReconstructionFilter,
    /// sensor diagonal in meters
    sensor_diagonal: f32,
    sensor: PixelSensor,
    file_name: String,
}

#[derive(Debug)]
pub struct RgbFilm {
    full_resolution: U64Vec2,
    pixel_bounds: Bounds2i,
    sensor: PixelSensor,
    sensor_diagonal: f32,
    file_name: String,
    color_space: Arc<RgbColorSpace>,
    max_component_value: f32,
    filter_integral: f32,
    write_fp16: bool,
    output_rgb_from_sensor_rgb: Mat3A,
    pixels: Vec<Pixel>,
    filter: ReconstructionFilter,
}

impl RgbFilm {
    pub fn new(
        parameters: FilmBaseParameters,
        color_space: Arc<RgbColorSpace>,
        max_component_value: f32,
        write_fp16: bool,
    ) -> Self {
        let filter_integral = parameters.filter.integral();
        let pixels = vec![Pixel::default(); parameters.pixel_bounds.area() as usize];
        let output_rgb_from_sensor_rgb =
            color_space.rgb_from_xyz * parameters.sensor.xyz_from_sensor_rgb().clone();
        RgbFilm {
            full_resolution: parameters.full_resolution,
            pixel_bounds: parameters.pixel_bounds,
            sensor: parameters.sensor,
            filter: parameters.filter,
            filter_integral,
            pixels,
            write_fp16,
            color_space,
            sensor_diagonal: parameters.sensor_diagonal,
            file_name: parameters.file_name,
            max_component_value,
            output_rgb_from_sensor_rgb,
        }
    }

    fn index(&self, location: U64Vec2) -> usize {
        let width = self.pixel_bounds.y_extent() as u64;
        (width * location.x + location.y) as usize
    }

    pub fn add_sample(
        &mut self,
        location: U64Vec2,
        sample: SampledSpectrum,
        lambda: SampledWavelengths,
        weight: f32,
    ) {
        let mut rgb = self.sensor.to_sensor_rgb(&sample, &lambda);
        let max = rgb.max();
        if max > self.max_component_value {
            rgb *= self.max_component_value / max;
        }

        let idx = self.index(location);
        let pixel = &mut self.pixels[idx];
        pixel.rgb_sum[0] += (rgb.r * weight) as f64;
        pixel.rgb_sum[1] += (rgb.g * weight) as f64;
        pixel.rgb_sum[2] += (rgb.b * weight) as f64;
        pixel.weight_sum += weight as f64;
    }
}

#[derive(Debug, Default, Clone)]
pub struct Pixel {
    pub rgb_sum: [f64; 3],
    pub weight_sum: f64,
}

#[derive(Debug)]
pub struct PixelSensor {
    r_bar: Spectrum,
    g_bar: Spectrum,
    b_bar: Spectrum,
    imaging_ratio: f32,
    xyz_from_sensor_rgb: Mat3A,
}

impl Default for PixelSensor {
    fn default() -> Self {
        PixelSensor::create(&S_RGB, 100.0, 6500.0, 1.0)
    }
}

impl PixelSensor {
    pub fn create(
        color_space: &RgbColorSpace,
        iso: f32,
        white_balance: f32,
        exposure_time: f32,
    ) -> PixelSensor {
        let imaging_ratio = exposure_time * iso / 100.0;
        let d_illum = NamedSpectra::d_illuminant(white_balance);

        PixelSensor::new(
            CIE_XYZ.x.clone(),
            CIE_XYZ.y.clone(),
            CIE_XYZ.z.clone(),
            color_space,
            d_illum,
            imaging_ratio,
        )
    }

    fn new(
        r_bar: Spectrum,
        g_bar: Spectrum,
        b_bar: Spectrum,
        color_space: &RgbColorSpace,
        sensor_illum: Spectrum,
        imaging_ratio: f32,
    ) -> Self {
        let rgb_camera: Vec<_> = SWATCH_REFLECTANCES
            .iter()
            .map(|refl| project_reflectance(refl, &sensor_illum, &r_bar, &g_bar, &b_bar))
            .collect();

        let sensor_white_g = inner_product(&sensor_illum, &g_bar);
        let sensor_white_y = inner_product(&sensor_illum, &CIE_XYZ.y);

        let xyz_output: Vec<_> = SWATCH_REFLECTANCES
            .iter()
            .map(|refl| {
                project_reflectance(
                    refl,
                    color_space.illuminant.as_ref(),
                    &CIE_XYZ.x,
                    &CIE_XYZ.y,
                    &CIE_XYZ.z,
                ) * (sensor_white_y / sensor_white_g)
            })
            .collect();

        let m = linear_least_squares(&rgb_camera, &xyz_output);

        PixelSensor {
            r_bar,
            g_bar,
            b_bar,
            imaging_ratio,
            xyz_from_sensor_rgb: m,
        }
    }

    fn with_xyz(color_space: &RgbColorSpace, sensor_illum: Spectrum, imaging_ratio: f32) -> Self {
        let source_white = Xyz::from(&sensor_illum).xy();
        let target_white = color_space.w;
        let white_balance = white_balance(source_white, target_white);

        PixelSensor {
            r_bar: CIE_XYZ.x.clone(),
            g_bar: CIE_XYZ.y.clone(),
            b_bar: CIE_XYZ.z.clone(),
            imaging_ratio,
            xyz_from_sensor_rgb: white_balance,
        }
    }

    pub fn to_sensor_rgb(&self, sample: &SampledSpectrum, lambda: &SampledWavelengths) -> Rgb {
        let l = sample.safe_div(lambda.pdf());
        Rgb {
            r: (self.r_bar.sample(lambda) * l).average(),
            g: (self.g_bar.sample(lambda) * l).average(),
            b: (self.b_bar.sample(lambda) * l).average(),
        } * self.imaging_ratio
    }

    pub fn xyz_from_sensor_rgb(&self) -> &Mat3A {
        &self.xyz_from_sensor_rgb
    }
}

fn project_reflectance(
    refl: &Spectrum,
    illum: &Spectrum,
    b1: &Spectrum,
    b2: &Spectrum,
    b3: &Spectrum,
) -> Vec3A {
    let mut result = Vec3A::ZERO;
    let mut g_integral = 0.0;

    for lambda in (LAMBDA_MIN as usize)..=(LAMBDA_MAX as usize) {
        let lambda = lambda as f32;
        g_integral += b2.evaluate(lambda) * illum.evaluate(lambda);
        result.x += b1.evaluate(lambda) * refl.evaluate(lambda) * illum.evaluate(lambda);
        result.y += b2.evaluate(lambda) * refl.evaluate(lambda) * illum.evaluate(lambda);
        result.z += b3.evaluate(lambda) * refl.evaluate(lambda) * illum.evaluate(lambda);
    }

    result / g_integral
}

fn white_balance(source_white: Vec2, target_white: Vec2) -> Mat3A {
    #[rustfmt::skip]
    const LMS_FROM_XYZ: Mat3A = Mat3A::from_cols_array(&[
        0.8951, 0.2664, -0.1614,
        -0.7502, 1.7135, 0.0367,
        0.0389, -0.0685, 1.0296,
    ]);

    #[rustfmt::skip]
    const XYZ_FROM_LMS: Mat3A = Mat3A::from_cols_array(&[
        0.986993, -0.147054, 0.159963,
        0.432305, 0.51836, 0.0492912,
        -0.00852866, 0.0400428, 0.968487,
    ]);

    let src_xyz = Vec3A::from(Xyz::from_xy(source_white));
    let dest_xyz = Vec3A::from(Xyz::from_xy(target_white));
    let src_lms = LMS_FROM_XYZ * src_xyz;
    let dest_lms = LMS_FROM_XYZ * dest_xyz;

    let lms_correct = Mat3A::from_diagonal(vec3(
        dest_lms.x / src_lms.x,
        dest_lms.y / src_lms.y,
        dest_lms.z / src_lms.z,
    ));

    XYZ_FROM_LMS * lms_correct * LMS_FROM_XYZ
}

fn load_swatch_reflectances() -> Vec<Spectrum> {
    vec![
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.055, 390.0, 0.058, 400.0, 0.061, 410.0, 0.062, 420.0, 0.062, 430.0, 0.062,
                440.0, 0.062, 450.0, 0.062, 460.0, 0.062, 470.0, 0.062, 480.0, 0.062, 490.0, 0.063,
                500.0, 0.065, 510.0, 0.070, 520.0, 0.076, 530.0, 0.079, 540.0, 0.081, 550.0, 0.084,
                560.0, 0.091, 570.0, 0.103, 580.0, 0.119, 590.0, 0.134, 600.0, 0.143, 610.0, 0.147,
                620.0, 0.151, 630.0, 0.158, 640.0, 0.168, 650.0, 0.179, 660.0, 0.188, 670.0, 0.190,
                680.0, 0.186, 690.0, 0.181, 700.0, 0.182, 710.0, 0.187, 720.0, 0.196, 730.0, 0.209,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.117, 390.0, 0.143, 400.0, 0.175, 410.0, 0.191, 420.0, 0.196, 430.0, 0.199,
                440.0, 0.204, 450.0, 0.213, 460.0, 0.228, 470.0, 0.251, 480.0, 0.280, 490.0, 0.309,
                500.0, 0.329, 510.0, 0.333, 520.0, 0.315, 530.0, 0.286, 540.0, 0.273, 550.0, 0.276,
                560.0, 0.277, 570.0, 0.289, 580.0, 0.339, 590.0, 0.420, 600.0, 0.488, 610.0, 0.525,
                620.0, 0.546, 630.0, 0.562, 640.0, 0.578, 650.0, 0.595, 660.0, 0.612, 670.0, 0.625,
                680.0, 0.638, 690.0, 0.656, 700.0, 0.678, 710.0, 0.700, 720.0, 0.717, 730.0, 0.734,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.130, 390.0, 0.177, 400.0, 0.251, 410.0, 0.306, 420.0, 0.324, 430.0, 0.330,
                440.0, 0.333, 450.0, 0.331, 460.0, 0.323, 470.0, 0.311, 480.0, 0.298, 490.0, 0.285,
                500.0, 0.269, 510.0, 0.250, 520.0, 0.231, 530.0, 0.214, 540.0, 0.199, 550.0, 0.185,
                560.0, 0.169, 570.0, 0.157, 580.0, 0.149, 590.0, 0.145, 600.0, 0.142, 610.0, 0.141,
                620.0, 0.141, 630.0, 0.141, 640.0, 0.143, 650.0, 0.147, 660.0, 0.152, 670.0, 0.154,
                680.0, 0.150, 690.0, 0.144, 700.0, 0.136, 710.0, 0.132, 720.0, 0.135, 730.0, 0.147,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.051, 390.0, 0.054, 400.0, 0.056, 410.0, 0.057, 420.0, 0.058, 430.0, 0.059,
                440.0, 0.060, 450.0, 0.061, 460.0, 0.062, 470.0, 0.063, 480.0, 0.065, 490.0, 0.067,
                500.0, 0.075, 510.0, 0.101, 520.0, 0.145, 530.0, 0.178, 540.0, 0.184, 550.0, 0.170,
                560.0, 0.149, 570.0, 0.133, 580.0, 0.122, 590.0, 0.115, 600.0, 0.109, 610.0, 0.105,
                620.0, 0.104, 630.0, 0.106, 640.0, 0.109, 650.0, 0.112, 660.0, 0.114, 670.0, 0.114,
                680.0, 0.112, 690.0, 0.112, 700.0, 0.115, 710.0, 0.120, 720.0, 0.125, 730.0, 0.130,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.144, 390.0, 0.198, 400.0, 0.294, 410.0, 0.375, 420.0, 0.408, 430.0, 0.421,
                440.0, 0.426, 450.0, 0.426, 460.0, 0.419, 470.0, 0.403, 480.0, 0.379, 490.0, 0.346,
                500.0, 0.311, 510.0, 0.281, 520.0, 0.254, 530.0, 0.229, 540.0, 0.214, 550.0, 0.208,
                560.0, 0.202, 570.0, 0.194, 580.0, 0.193, 590.0, 0.200, 600.0, 0.214, 610.0, 0.230,
                620.0, 0.241, 630.0, 0.254, 640.0, 0.279, 650.0, 0.313, 660.0, 0.348, 670.0, 0.366,
                680.0, 0.366, 690.0, 0.359, 700.0, 0.358, 710.0, 0.365, 720.0, 0.377, 730.0, 0.398,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.136, 390.0, 0.179, 400.0, 0.247, 410.0, 0.297, 420.0, 0.320, 430.0, 0.337,
                440.0, 0.355, 450.0, 0.381, 460.0, 0.419, 470.0, 0.466, 480.0, 0.510, 490.0, 0.546,
                500.0, 0.567, 510.0, 0.574, 520.0, 0.569, 530.0, 0.551, 540.0, 0.524, 550.0, 0.488,
                560.0, 0.445, 570.0, 0.400, 580.0, 0.350, 590.0, 0.299, 600.0, 0.252, 610.0, 0.221,
                620.0, 0.204, 630.0, 0.196, 640.0, 0.191, 650.0, 0.188, 660.0, 0.191, 670.0, 0.199,
                680.0, 0.212, 690.0, 0.223, 700.0, 0.232, 710.0, 0.233, 720.0, 0.229, 730.0, 0.229,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.054, 390.0, 0.054, 400.0, 0.053, 410.0, 0.054, 420.0, 0.054, 430.0, 0.055,
                440.0, 0.055, 450.0, 0.055, 460.0, 0.056, 470.0, 0.057, 480.0, 0.058, 490.0, 0.061,
                500.0, 0.068, 510.0, 0.089, 520.0, 0.125, 530.0, 0.154, 540.0, 0.174, 550.0, 0.199,
                560.0, 0.248, 570.0, 0.335, 580.0, 0.444, 590.0, 0.538, 600.0, 0.587, 610.0, 0.595,
                620.0, 0.591, 630.0, 0.587, 640.0, 0.584, 650.0, 0.584, 660.0, 0.590, 670.0, 0.603,
                680.0, 0.620, 690.0, 0.639, 700.0, 0.655, 710.0, 0.663, 720.0, 0.663, 730.0, 0.667,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.122, 390.0, 0.164, 400.0, 0.229, 410.0, 0.286, 420.0, 0.327, 430.0, 0.361,
                440.0, 0.388, 450.0, 0.400, 460.0, 0.392, 470.0, 0.362, 480.0, 0.316, 490.0, 0.260,
                500.0, 0.209, 510.0, 0.168, 520.0, 0.138, 530.0, 0.117, 540.0, 0.104, 550.0, 0.096,
                560.0, 0.090, 570.0, 0.086, 580.0, 0.084, 590.0, 0.084, 600.0, 0.084, 610.0, 0.084,
                620.0, 0.084, 630.0, 0.085, 640.0, 0.090, 650.0, 0.098, 660.0, 0.109, 670.0, 0.123,
                680.0, 0.143, 690.0, 0.169, 700.0, 0.205, 710.0, 0.244, 720.0, 0.287, 730.0, 0.332,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.096, 390.0, 0.115, 400.0, 0.131, 410.0, 0.135, 420.0, 0.133, 430.0, 0.132,
                440.0, 0.130, 450.0, 0.128, 460.0, 0.125, 470.0, 0.120, 480.0, 0.115, 490.0, 0.110,
                500.0, 0.105, 510.0, 0.100, 520.0, 0.095, 530.0, 0.093, 540.0, 0.092, 550.0, 0.093,
                560.0, 0.096, 570.0, 0.108, 580.0, 0.156, 590.0, 0.265, 600.0, 0.399, 610.0, 0.500,
                620.0, 0.556, 630.0, 0.579, 640.0, 0.588, 650.0, 0.591, 660.0, 0.593, 670.0, 0.594,
                680.0, 0.598, 690.0, 0.602, 700.0, 0.607, 710.0, 0.609, 720.0, 0.609, 730.0, 0.610,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.092, 390.0, 0.116, 400.0, 0.146, 410.0, 0.169, 420.0, 0.178, 430.0, 0.173,
                440.0, 0.158, 450.0, 0.139, 460.0, 0.119, 470.0, 0.101, 480.0, 0.087, 490.0, 0.075,
                500.0, 0.066, 510.0, 0.060, 520.0, 0.056, 530.0, 0.053, 540.0, 0.051, 550.0, 0.051,
                560.0, 0.052, 570.0, 0.052, 580.0, 0.051, 590.0, 0.052, 600.0, 0.058, 610.0, 0.073,
                620.0, 0.096, 630.0, 0.119, 640.0, 0.141, 650.0, 0.166, 660.0, 0.194, 670.0, 0.227,
                680.0, 0.265, 690.0, 0.309, 700.0, 0.355, 710.0, 0.396, 720.0, 0.436, 730.0, 0.478,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.061, 390.0, 0.061, 400.0, 0.062, 410.0, 0.063, 420.0, 0.064, 430.0, 0.066,
                440.0, 0.069, 450.0, 0.075, 460.0, 0.085, 470.0, 0.105, 480.0, 0.139, 490.0, 0.192,
                500.0, 0.271, 510.0, 0.376, 520.0, 0.476, 530.0, 0.531, 540.0, 0.549, 550.0, 0.546,
                560.0, 0.528, 570.0, 0.504, 580.0, 0.471, 590.0, 0.428, 600.0, 0.381, 610.0, 0.347,
                620.0, 0.327, 630.0, 0.318, 640.0, 0.312, 650.0, 0.310, 660.0, 0.314, 670.0, 0.327,
                680.0, 0.345, 690.0, 0.363, 700.0, 0.376, 710.0, 0.381, 720.0, 0.378, 730.0, 0.379,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.063, 390.0, 0.063, 400.0, 0.063, 410.0, 0.064, 420.0, 0.064, 430.0, 0.064,
                440.0, 0.065, 450.0, 0.066, 460.0, 0.067, 470.0, 0.068, 480.0, 0.071, 490.0, 0.076,
                500.0, 0.087, 510.0, 0.125, 520.0, 0.206, 530.0, 0.305, 540.0, 0.383, 550.0, 0.431,
                560.0, 0.469, 570.0, 0.518, 580.0, 0.568, 590.0, 0.607, 600.0, 0.628, 610.0, 0.637,
                620.0, 0.640, 630.0, 0.642, 640.0, 0.645, 650.0, 0.648, 660.0, 0.651, 670.0, 0.653,
                680.0, 0.657, 690.0, 0.664, 700.0, 0.673, 710.0, 0.680, 720.0, 0.684, 730.0, 0.688,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.066, 390.0, 0.079, 400.0, 0.102, 410.0, 0.146, 420.0, 0.200, 430.0, 0.244,
                440.0, 0.282, 450.0, 0.309, 460.0, 0.308, 470.0, 0.278, 480.0, 0.231, 490.0, 0.178,
                500.0, 0.130, 510.0, 0.094, 520.0, 0.070, 530.0, 0.054, 540.0, 0.046, 550.0, 0.042,
                560.0, 0.039, 570.0, 0.038, 580.0, 0.038, 590.0, 0.038, 600.0, 0.038, 610.0, 0.039,
                620.0, 0.039, 630.0, 0.040, 640.0, 0.041, 650.0, 0.042, 660.0, 0.044, 670.0, 0.045,
                680.0, 0.046, 690.0, 0.046, 700.0, 0.048, 710.0, 0.052, 720.0, 0.057, 730.0, 0.065,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.052, 390.0, 0.053, 400.0, 0.054, 410.0, 0.055, 420.0, 0.057, 430.0, 0.059,
                440.0, 0.061, 450.0, 0.066, 460.0, 0.075, 470.0, 0.093, 480.0, 0.125, 490.0, 0.178,
                500.0, 0.246, 510.0, 0.307, 520.0, 0.337, 530.0, 0.334, 540.0, 0.317, 550.0, 0.293,
                560.0, 0.262, 570.0, 0.230, 580.0, 0.198, 590.0, 0.165, 600.0, 0.135, 610.0, 0.115,
                620.0, 0.104, 630.0, 0.098, 640.0, 0.094, 650.0, 0.092, 660.0, 0.093, 670.0, 0.097,
                680.0, 0.102, 690.0, 0.108, 700.0, 0.113, 710.0, 0.115, 720.0, 0.114, 730.0, 0.114,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.050, 390.0, 0.049, 400.0, 0.048, 410.0, 0.047, 420.0, 0.047, 430.0, 0.047,
                440.0, 0.047, 450.0, 0.047, 460.0, 0.046, 470.0, 0.045, 480.0, 0.044, 490.0, 0.044,
                500.0, 0.045, 510.0, 0.046, 520.0, 0.047, 530.0, 0.048, 540.0, 0.049, 550.0, 0.050,
                560.0, 0.054, 570.0, 0.060, 580.0, 0.072, 590.0, 0.104, 600.0, 0.178, 610.0, 0.312,
                620.0, 0.467, 630.0, 0.581, 640.0, 0.644, 650.0, 0.675, 660.0, 0.690, 670.0, 0.698,
                680.0, 0.706, 690.0, 0.715, 700.0, 0.724, 710.0, 0.730, 720.0, 0.734, 730.0, 0.738,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.058, 390.0, 0.054, 400.0, 0.052, 410.0, 0.052, 420.0, 0.053, 430.0, 0.054,
                440.0, 0.056, 450.0, 0.059, 460.0, 0.067, 470.0, 0.081, 480.0, 0.107, 490.0, 0.152,
                500.0, 0.225, 510.0, 0.336, 520.0, 0.462, 530.0, 0.559, 540.0, 0.616, 550.0, 0.650,
                560.0, 0.672, 570.0, 0.694, 580.0, 0.710, 590.0, 0.723, 600.0, 0.731, 610.0, 0.739,
                620.0, 0.746, 630.0, 0.752, 640.0, 0.758, 650.0, 0.764, 660.0, 0.769, 670.0, 0.771,
                680.0, 0.776, 690.0, 0.782, 700.0, 0.790, 710.0, 0.796, 720.0, 0.799, 730.0, 0.804,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.145, 390.0, 0.195, 400.0, 0.283, 410.0, 0.346, 420.0, 0.362, 430.0, 0.354,
                440.0, 0.334, 450.0, 0.306, 460.0, 0.276, 470.0, 0.248, 480.0, 0.218, 490.0, 0.190,
                500.0, 0.168, 510.0, 0.149, 520.0, 0.127, 530.0, 0.107, 540.0, 0.100, 550.0, 0.102,
                560.0, 0.104, 570.0, 0.109, 580.0, 0.137, 590.0, 0.200, 600.0, 0.290, 610.0, 0.400,
                620.0, 0.516, 630.0, 0.615, 640.0, 0.687, 650.0, 0.732, 660.0, 0.760, 670.0, 0.774,
                680.0, 0.783, 690.0, 0.793, 700.0, 0.803, 710.0, 0.812, 720.0, 0.817, 730.0, 0.825,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.108, 390.0, 0.141, 400.0, 0.192, 410.0, 0.236, 420.0, 0.261, 430.0, 0.286,
                440.0, 0.317, 450.0, 0.353, 460.0, 0.390, 470.0, 0.426, 480.0, 0.446, 490.0, 0.444,
                500.0, 0.423, 510.0, 0.385, 520.0, 0.337, 530.0, 0.283, 540.0, 0.231, 550.0, 0.185,
                560.0, 0.146, 570.0, 0.118, 580.0, 0.101, 590.0, 0.090, 600.0, 0.082, 610.0, 0.076,
                620.0, 0.074, 630.0, 0.073, 640.0, 0.073, 650.0, 0.074, 660.0, 0.076, 670.0, 0.077,
                680.0, 0.076, 690.0, 0.075, 700.0, 0.073, 710.0, 0.072, 720.0, 0.074, 730.0, 0.079,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.189, 390.0, 0.255, 400.0, 0.423, 410.0, 0.660, 420.0, 0.811, 430.0, 0.862,
                440.0, 0.877, 450.0, 0.884, 460.0, 0.891, 470.0, 0.896, 480.0, 0.899, 490.0, 0.904,
                500.0, 0.907, 510.0, 0.909, 520.0, 0.911, 530.0, 0.910, 540.0, 0.911, 550.0, 0.914,
                560.0, 0.913, 570.0, 0.916, 580.0, 0.915, 590.0, 0.916, 600.0, 0.914, 610.0, 0.915,
                620.0, 0.918, 630.0, 0.919, 640.0, 0.921, 650.0, 0.923, 660.0, 0.924, 670.0, 0.922,
                680.0, 0.922, 690.0, 0.925, 700.0, 0.927, 710.0, 0.930, 720.0, 0.930, 730.0, 0.933,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.171, 390.0, 0.232, 400.0, 0.365, 410.0, 0.507, 420.0, 0.567, 430.0, 0.583,
                440.0, 0.588, 450.0, 0.590, 460.0, 0.591, 470.0, 0.590, 480.0, 0.588, 490.0, 0.588,
                500.0, 0.589, 510.0, 0.589, 520.0, 0.591, 530.0, 0.590, 540.0, 0.590, 550.0, 0.590,
                560.0, 0.589, 570.0, 0.591, 580.0, 0.590, 590.0, 0.590, 600.0, 0.587, 610.0, 0.585,
                620.0, 0.583, 630.0, 0.580, 640.0, 0.578, 650.0, 0.576, 660.0, 0.574, 670.0, 0.572,
                680.0, 0.571, 690.0, 0.569, 700.0, 0.568, 710.0, 0.568, 720.0, 0.566, 730.0, 0.566,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.144, 390.0, 0.192, 400.0, 0.272, 410.0, 0.331, 420.0, 0.350, 430.0, 0.357,
                440.0, 0.361, 450.0, 0.363, 460.0, 0.363, 470.0, 0.361, 480.0, 0.359, 490.0, 0.358,
                500.0, 0.358, 510.0, 0.359, 520.0, 0.360, 530.0, 0.360, 540.0, 0.361, 550.0, 0.361,
                560.0, 0.360, 570.0, 0.362, 580.0, 0.362, 590.0, 0.361, 600.0, 0.359, 610.0, 0.358,
                620.0, 0.355, 630.0, 0.352, 640.0, 0.350, 650.0, 0.348, 660.0, 0.345, 670.0, 0.343,
                680.0, 0.340, 690.0, 0.338, 700.0, 0.335, 710.0, 0.334, 720.0, 0.332, 730.0, 0.331,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.105, 390.0, 0.131, 400.0, 0.163, 410.0, 0.180, 420.0, 0.186, 430.0, 0.190,
                440.0, 0.193, 450.0, 0.194, 460.0, 0.194, 470.0, 0.192, 480.0, 0.191, 490.0, 0.191,
                500.0, 0.191, 510.0, 0.192, 520.0, 0.192, 530.0, 0.192, 540.0, 0.192, 550.0, 0.192,
                560.0, 0.192, 570.0, 0.193, 580.0, 0.192, 590.0, 0.192, 600.0, 0.191, 610.0, 0.189,
                620.0, 0.188, 630.0, 0.186, 640.0, 0.184, 650.0, 0.182, 660.0, 0.181, 670.0, 0.179,
                680.0, 0.178, 690.0, 0.176, 700.0, 0.174, 710.0, 0.173, 720.0, 0.172, 730.0, 0.171,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.068, 390.0, 0.077, 400.0, 0.084, 410.0, 0.087, 420.0, 0.089, 430.0, 0.090,
                440.0, 0.092, 450.0, 0.092, 460.0, 0.091, 470.0, 0.090, 480.0, 0.090, 490.0, 0.090,
                500.0, 0.090, 510.0, 0.090, 520.0, 0.090, 530.0, 0.090, 540.0, 0.090, 550.0, 0.090,
                560.0, 0.090, 570.0, 0.090, 580.0, 0.090, 590.0, 0.089, 600.0, 0.089, 610.0, 0.088,
                620.0, 0.087, 630.0, 0.086, 640.0, 0.086, 650.0, 0.085, 660.0, 0.084, 670.0, 0.084,
                680.0, 0.083, 690.0, 0.083, 700.0, 0.082, 710.0, 0.081, 720.0, 0.081, 730.0, 0.081,
            ],
            false,
        ),
        PiecewiseLinear::from_interleaved(
            vec![
                380.0, 0.031, 390.0, 0.032, 400.0, 0.032, 410.0, 0.033, 420.0, 0.033, 430.0, 0.033,
                440.0, 0.033, 450.0, 0.033, 460.0, 0.032, 470.0, 0.032, 480.0, 0.032, 490.0, 0.032,
                500.0, 0.032, 510.0, 0.032, 520.0, 0.032, 530.0, 0.032, 540.0, 0.032, 550.0, 0.032,
                560.0, 0.032, 570.0, 0.032, 580.0, 0.032, 590.0, 0.032, 600.0, 0.032, 610.0, 0.032,
                620.0, 0.032, 630.0, 0.032, 640.0, 0.032, 650.0, 0.032, 660.0, 0.032, 670.0, 0.032,
                680.0, 0.032, 690.0, 0.032, 700.0, 0.032, 710.0, 0.032, 720.0, 0.032, 730.0, 0.033,
            ],
            false,
        ),
    ]
    .into_iter()
    .map(From::from)
    .collect()
}

#[cfg(test)]
mod test {
    use glam::{i64vec2, u64vec2, vec2};

    use super::{FilmBaseParameters, PixelSensor, RgbFilm};
    use crate::camera::Bounds2i;
    use crate::color::colorspace::{RgbColorSpace, S_RGB};
    use crate::color::rgb::Rgb;
    use crate::filter::{Gaussian, ReconstructionFilter};
    use crate::random::random;
    use crate::spectrum::{
        HasWavelength, RgbAlbedo, SampledSpectrum, SampledWavelengths, Spectrum,
    };

    fn get_rgb_sample(
        r: f32,
        g: f32,
        b: f32,
        color_space: &RgbColorSpace,
    ) -> (SampledSpectrum, SampledWavelengths) {
        let lambda = SampledWavelengths::sample_uniform(random());
        let spectrum: Spectrum = RgbAlbedo::with_color_space(color_space, Rgb { r, g, b }).into();
        let sample = spectrum.sample(&lambda);

        (sample, lambda)
    }

    #[test]
    fn create_pixel_sensor() {
        let sensor = PixelSensor::create(&S_RGB, 100.0, 6500.0, 1.0);
        let (sample, lambda) = get_rgb_sample(0.9, 0.1, 0.1, S_RGB.as_ref());
        let response = sensor.to_sensor_rgb(&sample, &lambda);
        assert!(response.r >= 0.9);
        assert!(response.g >= 0.1);
        assert!(response.b >= 0.1);
    }

    #[test]
    fn test_add_samples() {
        let parameters = FilmBaseParameters {
            full_resolution: u64vec2(400, 300),
            file_name: "file.png".into(),
            filter: ReconstructionFilter::Gaussian(Gaussian::new(vec2(1.0, 1.0), 1.0, 1.0, 1.0)),
            sensor: PixelSensor::default(),
            sensor_diagonal: 0.036,
            pixel_bounds: Bounds2i::new(i64vec2(50, 50), i64vec2(200, 200)),
        };

        let mut film = RgbFilm::new(
            parameters,
            // TODO move to Arc in the Lazy<T>?
            S_RGB.clone(),
            std::f32::INFINITY,
            false,
        );

        for i in 50..200 {
            for j in 50..200 {
                let (sample, lambda) = get_rgb_sample(0.9, 0.1, 0.1, S_RGB.as_ref());
                film.add_sample(u64vec2(i, j), sample, lambda, 1.0);
            }
        }

        for pixel in film.pixels {
            assert!(pixel.rgb_sum.iter().all(|f| *f >= 0.0));
            assert_eq!(pixel.weight_sum, 1.0);
        }
    }
}
