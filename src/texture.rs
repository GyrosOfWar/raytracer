use std::sync::Arc;

use enum_dispatch::enum_dispatch;

use crate::vec3::Point3;

#[derive(Debug, Clone, Copy)]
pub struct TextureCoordinates {
    pub u: f32,
    pub v: f32,
}

#[enum_dispatch]
pub trait HasColorValue: Send + Sync {
    fn value_at(&self, coords: TextureCoordinates, p: Point3<f32>) -> Point3<f32>;
}

#[derive(Debug)]
pub struct SolidColor {
    pub albedo: Point3<f32>,
}

impl HasColorValue for SolidColor {
    fn value_at(&self, _: TextureCoordinates, _: Point3<f32>) -> Point3<f32> {
        self.albedo
    }
}

#[derive(Debug)]
pub struct Checkerboard {
    inv_scale: f32,
    even: Arc<Texture>,
    odd: Arc<Texture>,
}

impl Checkerboard {
    pub fn new(inv_scale: f32, even: Arc<Texture>, odd: Arc<Texture>) -> Self {
        Checkerboard {
            inv_scale,
            even,
            odd,
        }
    }
}

impl HasColorValue for Checkerboard {
    fn value_at(&self, coords: TextureCoordinates, p: Point3<f32>) -> Point3<f32> {
        let x = (self.inv_scale * p.x).floor() as i64;
        let y = (self.inv_scale * p.y).floor() as i64;
        let z = (self.inv_scale * p.z).floor() as i64;

        let is_even = (x + y + z) % 2 == 0;
        if is_even {
            self.even.value_at(coords, p)
        } else {
            self.odd.value_at(coords, p)
        }
    }
}

#[enum_dispatch(HasColorValue)]
#[derive(Debug)]
pub enum Texture {
    SolidColor(SolidColor),
    Checkerboard(Checkerboard),
}

pub fn solid(albedo: Point3<f32>) -> Arc<Texture> {
    Arc::new(Texture::SolidColor(SolidColor { albedo }))
}

pub fn checkerboard(inv_scale: f32, even: Arc<Texture>, solid: Arc<Texture>) -> Arc<Texture> {
    Arc::new(Texture::Checkerboard(Checkerboard::new(
        inv_scale, even, solid,
    )))
}
