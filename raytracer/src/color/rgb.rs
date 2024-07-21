use glam::Vec3A;

use crate::math::evaluate_polynomial;
use crate::spectrum::{HasWavelength, LAMBDA_MAX, LAMBDA_MIN};

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

#[cfg(test)]
mod tests {
    use crate::color::rgb::Rgb;

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
}
