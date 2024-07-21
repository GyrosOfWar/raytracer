use std::fmt::Debug;
use std::ops::Mul;

use color_eyre::Result;
use enum_dispatch::enum_dispatch;
use once_cell::sync::Lazy;
use ordered_float::OrderedFloat;

use crate::color::colorspace::RgbColorSpace;
use crate::color::rgb::{Rgb, RgbSigmoidPolynomial};
use crate::color::xyz::{Xyz, CIE_XYZ, CIE_Y_INTEGRAL};
use crate::math::lerp;
use crate::range::Range;
use crate::util::{self, is_sorted};

pub const LAMBDA_MIN: f32 = 360.0;
pub const LAMBDA_MAX: f32 = 830.0;
pub const VISIBLE_RANGE: Range = Range::new(LAMBDA_MIN, LAMBDA_MAX);

#[enum_dispatch]
pub trait HasWavelength: Send + Sync + Debug {
    // see spectrum.h operator()
    fn evaluate(&self, lambda: f32) -> f32;

    fn max_value(&self) -> f32;

    fn sample(&self, lambda: &SampledWavelengths) -> SampledSpectrum {
        let spectrum: Vec<_> = lambda
            .lambda
            .into_iter()
            .map(|w| self.evaluate(w))
            .collect();
        SampledSpectrum::from_array(spectrum.try_into().expect("must have correct length"))
    }
}

#[enum_dispatch(HasWavelength)]
#[derive(Debug, Clone)]
pub enum Spectrum {
    Constant(Constant),
    DenselSampled(DenselySampled),
    PiecewiseLinear(PiecewiseLinear),
    Blackbody(Blackbody),
    RgbAlbedo(RgbAlbedo),
    RgbUnbounded(RgbUnbounded),
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub c: f32,
}

impl HasWavelength for Constant {
    fn evaluate(&self, _: f32) -> f32 {
        self.c
    }

    fn max_value(&self) -> f32 {
        self.c
    }
}

#[derive(Debug, Clone)]
pub struct DenselySampled {
    lambda_min: usize,
    lambda_max: usize,
    values: Vec<f32>,
}

impl DenselySampled {
    pub fn new(values: Vec<f32>) -> Self {
        DenselySampled {
            lambda_max: LAMBDA_MAX as usize,
            lambda_min: LAMBDA_MIN as usize,
            values,
        }
    }

    pub fn from_spectrum_in_range(spec: Spectrum, lambda_min: usize, lambda_max: usize) -> Self {
        let mut values = vec![0.0; lambda_max - lambda_min];
        for lambda in lambda_min..lambda_max {
            values[lambda - lambda_min] = spec.evaluate(lambda as f32);
        }
        DenselySampled {
            values,
            lambda_max,
            lambda_min,
        }
    }

    pub fn from_spectrum(spec: Spectrum) -> Self {
        Self::from_spectrum_in_range(spec, LAMBDA_MIN as usize, LAMBDA_MAX as usize)
    }
}

impl HasWavelength for DenselySampled {
    fn evaluate(&self, lambda: f32) -> f32 {
        let offset = lambda.round() as usize - self.lambda_min;
        if offset >= self.values.len() {
            0.0
        } else {
            self.values[offset]
        }
    }

    fn max_value(&self) -> f32 {
        util::max_value(&self.values)
    }
}

#[derive(Debug, Clone)]
pub struct PiecewiseLinear {
    lambdas: Vec<f32>,
    values: Vec<f32>,
}

impl PiecewiseLinear {
    pub fn new(mut lambdas: Vec<f32>, values: Vec<f32>) -> Self {
        if !is_sorted(&lambdas) {
            lambdas.sort_by_key(|f| OrderedFloat(*f));
        }
        Self { lambdas, values }
    }

    pub fn from_interleaved(samples: Vec<f32>, normalize: bool) -> Self {
        assert!(samples.len() % 2 == 0);

        let n = samples.len() / 2;
        let mut lambda = vec![];
        let mut v = vec![];

        if samples[0] > LAMBDA_MIN {
            lambda.push(LAMBDA_MIN - 1.0);
            v.push(samples[0]);
        }

        for i in 0..n {
            lambda.push(samples[2 * i]);
            v.push(samples[2 * i + 1]);
        }

        if *lambda.last().unwrap() < LAMBDA_MAX {
            lambda.push(LAMBDA_MAX + 1.0);
            let last = v.last().unwrap();
            v.push(*last);
        }

        let mut spectrum = PiecewiseLinear::new(lambda, v);
        if normalize {
            spectrum.scale(CIE_Y_INTEGRAL / inner_product(&spectrum, &CIE_XYZ.y));
        }

        spectrum
    }

    pub fn scale(&mut self, factor: f32) {
        for v in &mut self.values {
            *v = *v * factor;
        }
    }
}

impl HasWavelength for PiecewiseLinear {
    fn evaluate(&self, lambda: f32) -> f32 {
        if self.lambdas.is_empty()
            || lambda < *self.lambdas.first().unwrap()
            || lambda > *self.lambdas.last().unwrap()
        {
            0.0
        } else {
            let o = util::find_interval(&self.lambdas, lambda);
            let t = (lambda - self.lambdas[o]) / (self.lambdas[o + 1] - self.lambdas[o]);
            lerp(t, self.values[o], self.values[o + 1])
        }
    }

    fn max_value(&self) -> f32 {
        util::max_value(&self.values)
    }
}

#[derive(Debug, Clone)]
pub struct Blackbody {
    kelvin: f32,
    normalization_factor: f32,
}

impl Blackbody {
    pub fn new(kelvin: f32) -> Self {
        let lambda_max = 2.8977721e-3 / kelvin;
        Self {
            kelvin,
            normalization_factor: 1.0 / blackbody(lambda_max * 1e9, kelvin),
        }
    }
}

impl HasWavelength for Blackbody {
    fn evaluate(&self, lambda: f32) -> f32 {
        blackbody(lambda, self.kelvin) * self.normalization_factor
    }

    fn max_value(&self) -> f32 {
        1.0
    }
}

fn blackbody(lambda: f32, kelvin: f32) -> f32 {
    const C: f32 = 299792458.0;
    const H: f32 = 6.62606957e-34;
    const KB: f32 = 1.3806488e-23;

    if kelvin <= 0.0 {
        0.0
    } else {
        let l = lambda * 1e-9;
        let exp = ((H * C) / (l * KB * kelvin) - 1.0).exp();
        (2.0 * H * C * C) / (l.powi(5) * exp)
    }
}

#[derive(Debug, Clone)]
pub struct RgbAlbedo {
    coefficients: RgbSigmoidPolynomial,
}

impl RgbAlbedo {
    pub fn new(coefficients: RgbSigmoidPolynomial) -> Self {
        RgbAlbedo { coefficients }
    }
}

impl HasWavelength for RgbAlbedo {
    fn evaluate(&self, lambda: f32) -> f32 {
        self.coefficients.evaluate(lambda)
    }

    fn max_value(&self) -> f32 {
        self.coefficients.max_value()
    }
}

#[derive(Debug, Clone)]
pub struct RgbUnbounded {
    coefficients: RgbSigmoidPolynomial,
    scale: f32,
}

impl RgbUnbounded {
    pub fn new(coefficients: RgbSigmoidPolynomial, scale: f32) -> Self {
        RgbUnbounded {
            coefficients,
            scale,
        }
    }
}

impl HasWavelength for RgbUnbounded {
    fn evaluate(&self, lambda: f32) -> f32 {
        self.scale * self.coefficients.evaluate(lambda)
    }

    fn max_value(&self) -> f32 {
        self.scale * self.coefficients.max_value()
    }
}

pub struct RgbIlluminant {}

impl RgbIlluminant {}

pub fn inner_product(f: &impl HasWavelength, g: &impl HasWavelength) -> f32 {
    let mut integral = 0.0;
    for lambda in (LAMBDA_MIN as usize)..(LAMBDA_MAX as usize) {
        integral += f.evaluate(lambda as f32) * g.evaluate(lambda as f32);
    }

    integral
}

pub const N_SPECTRUM_SAMPLES: usize = 4;

#[derive(Debug, Copy, Clone)]
pub struct SampledSpectrum {
    samples: [f32; N_SPECTRUM_SAMPLES],
}

impl SampledSpectrum {
    pub fn from_slice(slice: &[f32]) -> Result<Self> {
        Ok(SampledSpectrum {
            samples: slice.try_into()?,
        })
    }

    pub fn from_array(array: [f32; N_SPECTRUM_SAMPLES]) -> Self {
        SampledSpectrum { samples: array }
    }

    pub fn is_zero(&self) -> bool {
        self.samples.iter().copied().all(|n| n == 0.0)
    }

    pub fn safe_div(self, rhs: SampledSpectrum) -> Self {
        let mut samples = self.samples;
        for (i, value) in rhs.samples.iter().enumerate() {
            samples[i] = if *value != 0.0 {
                samples[i] / value
            } else {
                0.0
            };
        }
        SampledSpectrum { samples }
    }

    pub fn average(&self) -> f32 {
        let sum: f32 = self.samples.iter().sum();
        sum / self.samples.len() as f32
    }

    pub fn to_xyz(&self, value: SampledWavelengths) -> Xyz {
        let x = CIE_XYZ.x.sample(&value);
        let y = CIE_XYZ.y.sample(&value);
        let z = CIE_XYZ.z.sample(&value);
        let pdf = value.pdf();
        let xyz = Xyz {
            x: (x * *self).safe_div(pdf).average(),
            y: (y * *self).safe_div(pdf).average(),
            z: (z * *self).safe_div(pdf).average(),
        };

        xyz / CIE_Y_INTEGRAL
    }

    pub fn to_rgb(&self, wavelengths: SampledWavelengths, color_space: &RgbColorSpace) -> Rgb {
        let xyz = self.to_xyz(wavelengths);
        color_space.to_rgb(xyz)
    }
}

impl Mul for SampledSpectrum {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut values = self.samples;
        for (idx, value) in rhs.samples.iter().enumerate() {
            values[idx] *= value;
        }
        SampledSpectrum { samples: values }
    }
}

#[derive(Debug)]
pub struct SampledWavelengths {
    pub lambda: [f32; N_SPECTRUM_SAMPLES],
    pub pdf: [f32; N_SPECTRUM_SAMPLES],
}

impl SampledWavelengths {
    pub fn sample_uniform(u: f32) -> Self {
        Self::sample_uniform_in_range(u, LAMBDA_MIN, LAMBDA_MAX)
    }

    pub fn sample_uniform_in_range(u: f32, lambda_min: f32, lambda_max: f32) -> Self {
        let mut lambda = [0.0; N_SPECTRUM_SAMPLES];
        lambda[0] = lerp(u, lambda_min, lambda_max);
        let delta = (lambda_max - lambda_min) / N_SPECTRUM_SAMPLES as f32;
        for i in 1..N_SPECTRUM_SAMPLES {
            lambda[i] = lambda[i - 1] + delta;
            if lambda[i] > lambda_max {
                lambda[i] = lambda_min + (lambda[i] - lambda_max);
            }
        }

        let pdf = [1.0 / (lambda_max - lambda_min); N_SPECTRUM_SAMPLES];

        Self { lambda, pdf }
    }

    pub fn pdf(&self) -> SampledSpectrum {
        SampledSpectrum::from_array(self.pdf)
    }
}

pub static NAMED_SPECTRA: Lazy<NamedSpectra> = Lazy::new(NamedSpectra::new);

pub struct NamedSpectra {
    pub std_illum_d65: Spectrum,
    pub illum_aces_d60: Spectrum,
    // todo
}

impl NamedSpectra {
    pub fn new() -> Self {
        const CIE_ILLUM_D65: [f32; 214] = [
            300.0, 0.034100, 305.0, 1.664300, 310.0, 3.294500, 315.0, 11.765200, 320.0, 20.236000,
            325.0, 28.644699, 330.0, 37.053501, 335.0, 38.501099, 340.0, 39.948799, 345.0,
            42.430199, 350.0, 44.911701, 355.0, 45.775002, 360.0, 46.638302, 365.0, 49.363701,
            370.0, 52.089100, 375.0, 51.032299, 380.0, 49.975498, 385.0, 52.311798, 390.0,
            54.648201, 395.0, 68.701500, 400.0, 82.754898, 405.0, 87.120399, 410.0, 91.486000,
            415.0, 92.458900, 420.0, 93.431801, 425.0, 90.056999, 430.0, 86.682297, 435.0,
            95.773598, 440.0, 104.864998, 445.0, 110.935997, 450.0, 117.008003, 455.0, 117.410004,
            460.0, 117.811996, 465.0, 116.335999, 470.0, 114.861000, 475.0, 115.391998, 480.0,
            115.922997, 485.0, 112.366997, 490.0, 108.810997, 495.0, 109.082001, 500.0, 109.353996,
            505.0, 108.578003, 510.0, 107.802002, 515.0, 106.295998, 520.0, 104.790001, 525.0,
            106.238998, 530.0, 107.689003, 535.0, 106.046997, 540.0, 104.404999, 545.0, 104.224998,
            550.0, 104.045998, 555.0, 102.023003, 560.0, 100.0, 565.0, 98.167099, 570.0, 96.334198,
            575.0, 96.061096, 580.0, 95.788002, 585.0, 92.236801, 590.0, 88.685600, 595.0,
            89.345901, 600.0, 90.006203, 605.0, 89.802597, 610.0, 89.599098, 615.0, 88.648903,
            620.0, 87.698700, 625.0, 85.493599, 630.0, 83.288597, 635.0, 83.493896, 640.0,
            83.699203, 645.0, 81.862999, 650.0, 80.026802, 655.0, 80.120697, 660.0, 80.214600,
            665.0, 81.246201, 670.0, 82.277802, 675.0, 80.280998, 680.0, 78.284203, 685.0,
            74.002701, 690.0, 69.721298, 695.0, 70.665199, 700.0, 71.609100, 705.0, 72.978996,
            710.0, 74.348999, 715.0, 67.976501, 720.0, 61.604000, 725.0, 65.744797, 730.0,
            69.885597, 735.0, 72.486298, 740.0, 75.086998, 745.0, 69.339798, 750.0, 63.592701,
            755.0, 55.005402, 760.0, 46.418201, 765.0, 56.611801, 770.0, 66.805397, 775.0,
            65.094101, 780.0, 63.382801, 785.0, 63.843399, 790.0, 64.304001, 795.0, 61.877899,
            800.0, 59.451900, 805.0, 55.705399, 810.0, 51.959000, 815.0, 54.699799, 820.0,
            57.440601, 825.0, 58.876499, 830.0, 60.312500,
        ];
        const ILLUM_ACES_D60: [f32; 214] = [
            300.0, 0.02928, 305.0, 1.28964, 310.0, 2.55, 315.0, 9.0338, 320.0, 15.5176, 325.0,
            21.94705, 330.0, 28.3765, 335.0, 29.93335, 340.0, 31.4902, 345.0, 33.75765, 350.0,
            36.0251, 355.0, 37.2032, 360.0, 38.3813, 365.0, 40.6445, 370.0, 42.9077, 375.0,
            42.05735, 380.0, 41.207, 385.0, 43.8121, 390.0, 46.4172, 395.0, 59.26285, 400.0,
            72.1085, 405.0, 76.1756, 410.0, 80.2427, 415.0, 81.4878, 420.0, 82.7329, 425.0,
            80.13505, 430.0, 77.5372, 435.0, 86.5577, 440.0, 95.5782, 445.0, 101.72045, 450.0,
            107.8627, 455.0, 108.67115, 460.0, 109.4796, 465.0, 108.5873, 470.0, 107.695, 475.0,
            108.6598, 480.0, 109.6246, 485.0, 106.6426, 490.0, 103.6606, 495.0, 104.42795, 500.0,
            105.1953, 505.0, 104.7974, 510.0, 104.3995, 515.0, 103.45635, 520.0, 102.5132, 525.0,
            104.2813, 530.0, 106.0494, 535.0, 104.67885, 540.0, 103.3083, 545.0, 103.4228, 550.0,
            103.5373, 555.0, 101.76865, 560.0, 100.0, 565.0, 98.3769, 570.0, 96.7538, 575.0,
            96.73515, 580.0, 96.7165, 585.0, 93.3013, 590.0, 89.8861, 595.0, 90.91705, 600.0,
            91.948, 605.0, 91.98965, 610.0, 92.0313, 615.0, 91.3008, 620.0, 90.5703, 625.0,
            88.5077, 630.0, 86.4451, 635.0, 86.9551, 640.0, 87.4651, 645.0, 85.6558, 650.0,
            83.8465, 655.0, 84.20755, 660.0, 84.5686, 665.0, 85.9432, 670.0, 87.3178, 675.0,
            85.3068, 680.0, 83.2958, 685.0, 78.66005, 690.0, 74.0243, 695.0, 75.23535, 700.0,
            76.4464, 705.0, 77.67465, 710.0, 78.9029, 715.0, 72.12575, 720.0, 65.3486, 725.0,
            69.6609, 730.0, 73.9732, 735.0, 76.6802, 740.0, 79.3872, 745.0, 73.28855, 750.0,
            67.1899, 755.0, 58.18595, 760.0, 49.182, 765.0, 59.9723, 770.0, 70.7626, 775.0,
            68.9039, 780.0, 67.0452, 785.0, 67.5469, 790.0, 68.0486, 795.0, 65.4631, 800.0,
            62.8776, 805.0, 58.88595, 810.0, 54.8943, 815.0, 57.8066, 820.0, 60.7189, 825.0,
            62.2491, 830.0, 63.7793,
        ];

        NamedSpectra {
            std_illum_d65: PiecewiseLinear::from_interleaved(CIE_ILLUM_D65.to_vec(), true).into(),
            illum_aces_d60: PiecewiseLinear::from_interleaved(ILLUM_ACES_D60.to_vec(), true).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HasWavelength, SampledWavelengths, NAMED_SPECTRA};
    use crate::spectrum::{LAMBDA_MAX, LAMBDA_MIN, N_SPECTRUM_SAMPLES};

    #[test]
    fn test_sampled_wavelengths_uniform() {
        let u = 0.0;
        let wavelength = SampledWavelengths::sample_uniform(u);
        let delta = (LAMBDA_MAX - LAMBDA_MIN) / N_SPECTRUM_SAMPLES as f32;
        assert_eq!(
            wavelength.lambda,
            [
                360.0,
                360.0 + delta,
                360.0 + (delta * 2.0),
                360.0 + (delta * 3.0)
            ]
        );
    }

    #[test]
    fn test_named_spectra() {
        for lambda in 360..830 {
            let value = NAMED_SPECTRA.illum_aces_d60.evaluate(lambda as f32);
            assert!(value > 0.0);

            let value = NAMED_SPECTRA.std_illum_d65.evaluate(lambda as f32);
            assert!(value > 0.0);
        }
    }
}
