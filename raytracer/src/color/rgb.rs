use std::ops::{Div, Mul, MulAssign};

use crate::math::{evaluate_polynomial, square};
use crate::spectrum::{HasWavelength, LAMBDA_MAX, LAMBDA_MIN};
use crate::vec::{Mat3, Vec3, VectorLike};

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<Vec3> for Rgb {
    fn from(value: Vec3) -> Self {
        Rgb::new(value.x, value.y, value.z)
    }
}

impl Mul<f32> for Rgb {
    type Output = Rgb;

    fn mul(self, rhs: f32) -> Self::Output {
        Rgb::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl MulAssign<f32> for Rgb {
    fn mul_assign(&mut self, rhs: f32) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}

impl Mul<Rgb> for Mat3 {
    type Output = Rgb;

    fn mul(self, rhs: Rgb) -> Self::Output {
        Rgb::from(self * Vec3::from(rhs))
    }
}

impl Div<f32> for Rgb {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Rgb::new(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}

impl From<Rgb> for Vec3 {
    fn from(value: Rgb) -> Self {
        Vec3::new(value.r, value.g, value.b)
    }
}

impl VectorLike<3, f32> for Rgb {
    fn component(&self, index: usize) -> f32 {
        self.component(index as u8)
    }

    fn data(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    fn from_data(data: [f32; 3]) -> Self {
        Rgb {
            r: data[0],
            g: data[1],
            b: data[2],
        }
    }
}

impl Rgb {
    pub const ZERO: Rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    pub fn new(r: f32, g: f32, b: f32) -> Rgb {
        Rgb { r, g, b }
    }

    pub fn max(&self) -> f32 {
        self.component(self.max_component_index())
    }

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

    pub fn clamp_zero(self) -> Rgb {
        Rgb {
            r: self.r.max(0.0),
            g: self.g.max(0.0),
            b: self.b.max(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RgbSigmoidPolynomial {
    pub c0: f32,
    pub c1: f32,
    pub c2: f32,
}

// implements the spectrum interface for convenience,
// but is itself not a spectrum, so left out of the Spectrum enum
impl HasWavelength for RgbSigmoidPolynomial {
    fn evaluate(&self, lambda: f32) -> f32 {
        let result = sigmoid(evaluate_polynomial(&[self.c0, self.c1, self.c2], lambda));

        result
    }

    fn max_value(&self) -> f32 {
        let result = self.evaluate(360.0).max(self.evaluate(830.0));
        let lambda = -self.c1 / (2.0 * self.c0);
        if (LAMBDA_MIN..=LAMBDA_MAX).contains(&lambda) {
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
        0.5 + x / (2.0 * f32::sqrt(1.0 + square(x)))
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
