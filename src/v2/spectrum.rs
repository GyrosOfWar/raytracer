use crate::math::lerp;
use crate::v2::util;
use crate::Result;
use enum_dispatch::enum_dispatch;
use ordered_float::OrderedFloat;
use std::fmt::Debug;

const LAMBDA_MAX: f32 = 830.0;
const LAMBDA_MIN: f32 = 360.0;

#[enum_dispatch]
pub trait HasWavelength: Send + Sync + Debug {
    // see spectrum.h operator()
    fn evaluate(&self, lambda: f32) -> f32;

    fn max_value(&self) -> f32;
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

// todo
pub struct SampledWavelengths {}

#[cfg(test)]
mod tests {}
