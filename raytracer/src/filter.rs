use std::f32::consts::PI;

use enum_dispatch::enum_dispatch;

use crate::math::square;
use crate::vec2::Vec2;

#[enum_dispatch(Filter)]
#[derive(Debug)]
pub enum ReconstructionFilter {
    Gaussian(Gaussian),
}

#[enum_dispatch]
pub trait Filter {
    fn radius(&self) -> Vec2;
    fn evaluate(&self, point: Vec2) -> f32;
    fn integral(&self) -> f32;
    fn sample(&self, point: Vec2) -> FilterSample;
}

#[derive(Debug)]
pub struct FilterSample {
    pub p: Vec2,
    pub weight: f32,
}

#[derive(Debug)]
pub struct Gaussian {
    radius: Vec2,
    sigma: f32,
    exp_x: f32,
    exp_y: f32,
}

impl Gaussian {
    pub fn new(radius: Vec2, sigma: f32, exp_x: f32, exp_y: f32) -> Self {
        Gaussian {
            radius,
            sigma,
            exp_x,
            exp_y,
        }
    }
}

impl Filter for Gaussian {
    fn radius(&self) -> Vec2 {
        self.radius
    }

    fn evaluate(&self, p: Vec2) -> f32 {
        let a = (gaussian(p.x, 0.0, self.sigma) - self.exp_x).max(0.0);
        let b = (gaussian(p.y, 0.0, self.sigma) - self.exp_y).max(0.0);
        a * b
    }

    fn integral(&self) -> f32 {
        let a = gaussian_integral(
            -self.radius.x,
            self.radius.x,
            0.0,
            self.sigma - 2.0 * self.radius.x * self.exp_x,
        );
        let b = gaussian_integral(
            -self.radius.y,
            self.radius.y,
            0.0,
            self.sigma - 2.0 * self.radius.y * self.exp_y,
        );

        a * b
    }

    fn sample(&self, point: Vec2) -> FilterSample {
        todo!()
    }
}

fn gaussian(x: f32, mu: f32, sigma: f32) -> f32 {
    1.0 / (2.0 * PI * sigma * sigma).sqrt() * (-square(x - mu) / (2.0 * sigma * sigma)).exp()
}

fn gaussian_integral(x0: f32, x1: f32, mu: f32, sigma: f32) -> f32 {
    let sigma_root_2 = sigma * 1.414213562373095;
    return 0.5 * (libm::erff((mu - x0) / sigma_root_2) - libm::erff((mu - x1) / sigma_root_2));
}
