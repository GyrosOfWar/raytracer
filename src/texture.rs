use std::{fmt, path::Path, sync::Arc};

use enum_dispatch::enum_dispatch;
use image::{DynamicImage, GenericImageView, ImageError};

use crate::{range::Range, vec3::Point3};

#[derive(Debug, Clone, Copy, Default)]
pub struct TextureCoordinates {
    pub u: f32,
    pub v: f32,
}

impl TextureCoordinates {
    pub fn clamp01(self) -> Self {
        TextureCoordinates {
            u: Range::UNIT.clamp(self.u),
            v: Range::UNIT.clamp(self.v),
        }
    }
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

pub struct Image {
    image: DynamicImage,
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.image.width())
            .field("height", &self.image.height())
            .finish()
    }
}

impl Image {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let image = image::open(path)?;
        Ok(Image { image })
    }
}

impl HasColorValue for Image {
    fn value_at(&self, coords: TextureCoordinates, _: Point3<f32>) -> Point3<f32> {
        if self.image.height() == 0 {
            return Point3::new(0.0, 1.0, 1.0);
        }
        let mut coords = coords.clamp01();
        // Flip V to image coordinates
        coords.v = 1.0 - coords.v;

        let i = (coords.u * self.image.width() as f32) as u32;
        let j = (coords.v * self.image.height() as f32) as u32;
        let pixel = self.image.get_pixel(i, j);
        let color_scale = 1.0 / 255.0;
        Point3::new(
            pixel.0[0] as f32 * color_scale,
            pixel.0[1] as f32 * color_scale,
            pixel.0[2] as f32 * color_scale,
        )
    }
}

#[enum_dispatch(HasColorValue)]
#[derive(Debug)]
pub enum Texture {
    SolidColor(SolidColor),
    Checkerboard(Checkerboard),
    Image(Image),
}

pub fn solid(albedo: Point3<f32>) -> Arc<Texture> {
    Arc::new(Texture::SolidColor(SolidColor { albedo }))
}

pub fn checkerboard(inv_scale: f32, even: Arc<Texture>, solid: Arc<Texture>) -> Arc<Texture> {
    Arc::new(Texture::Checkerboard(Checkerboard::new(
        inv_scale, even, solid,
    )))
}

pub fn image(path: impl AsRef<Path>) -> Arc<Texture> {
    Arc::new(Texture::Image(
        Image::load(path).expect("could not load image"),
    ))
}
