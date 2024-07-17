use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use ordered_float::OrderedFloat;

use crate::math::lerp;
use crate::v2::util;
use crate::Result;

const LAMBDA_MAX: f32 = 830.0;
const LAMBDA_MIN: f32 = 360.0;

#[enum_dispatch]
pub trait HasWavelength: Send + Sync + Debug {
    // see spectrum.h operator()
    fn evaluate(&self, lambda: f32) -> f32;

    fn max_value(&self) -> f32;

    fn sample(&self, lambda: SampledWavelengths) -> SampledSpectrum {
        let mut samples = [0.0; N_SPECTRUM_SAMPLES];
        for (idx, wavelength) in lambda.lambda.iter().copied().enumerate() {
            samples[idx] = self.evaluate(wavelength);
        }

        SampledSpectrum::from_array(samples)
    }
}

#[enum_dispatch(HasWavelength)]
#[derive(Debug)]
pub enum Spectrum {
    Constant(Constant),
    DenselySampled(DenselySampled),
    PiecewiseLinear(PiecewiseLinear),
    Blackbody(Blackbody),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct DenselySampled {
    lambda_min: i32,
    lambda_max: i32,
    values: Vec<f32>,
}

impl HasWavelength for DenselySampled {
    fn evaluate(&self, lambda: f32) -> f32 {
        let offset = lambda.round() as i32 - self.lambda_min;
        if offset < 0 || offset as usize >= self.values.len() {
            0.0
        } else {
            self.values[offset as usize]
        }
    }

    fn max_value(&self) -> f32 {
        util::max_value(&self.values)
    }
}

#[derive(Debug)]
pub struct PiecewiseLinear {
    lambdas: Vec<f32>,
    values: Vec<f32>,
}

impl HasWavelength for PiecewiseLinear {
    fn evaluate(&self, lambda: f32) -> f32 {
        if self.lambdas.is_empty()
            || lambda < *self.lambdas.first().unwrap()
            || lambda > *self.lambdas.last().unwrap()
        {
            0.0
        } else {
            let o = util::find_interval(self.lambdas.len(), |idx| self.lambdas[idx] <= lambda);
            let t = (lambda - self.lambdas[o]) / (self.lambdas[o + 1] - self.lambdas[o]);
            lerp(t, self.values[o], self.values[o + 1])
        }
    }

    fn max_value(&self) -> f32 {
        util::max_value(&self.values)
    }
}

#[derive(Debug)]
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

pub fn inner_product(f: Spectrum, g: Spectrum) -> f32 {
    let mut integral = 0.0;
    for lambda in (LAMBDA_MIN as usize)..(LAMBDA_MAX as usize) {
        integral += f.evaluate(lambda) * g.evaluate(lambda);
    }

    integral
}

pub const N_SPECTRUM_SAMPLES: usize = 4;

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
}

#[cfg(test)]
mod tests {
    use super::SampledWavelengths;
    use crate::v2::spectrum::{LAMBDA_MAX, LAMBDA_MIN, N_SPECTRUM_SAMPLES};

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
}
