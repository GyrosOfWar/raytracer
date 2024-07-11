use std::fmt;
use std::path::Path;
use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use image::{DynamicImage, GenericImageView, ImageError};

use crate::range::Range;
use crate::vec3::{Color, Point3, Vec3};

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

    pub fn from_array(uv: [f32; 2]) -> Self {
        TextureCoordinates { u: uv[0], v: uv[1] }
    }

    pub fn tri_lerp(uv0: Self, uv1: Self, uv2: Self, a: f32, b: f32) -> TextureCoordinates {
        TextureCoordinates {
            u: uv0.u * (1.0 - a - b) + uv1.u * a + uv2.u * b,
            v: uv0.v * (1.0 - a - b) + uv1.v * a + uv2.v * b,
        }
    }
}

#[enum_dispatch]
pub trait HasColorValue: Send + Sync {
    fn value_at(&self, coords: TextureCoordinates, p: Point3) -> Color;
}

#[derive(Debug)]
pub struct SolidColor {
    pub albedo: Color,
}

impl HasColorValue for SolidColor {
    fn value_at(&self, _: TextureCoordinates, _: Point3) -> Color {
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
    fn value_at(&self, coords: TextureCoordinates, p: Point3) -> Color {
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

impl Image {
    pub fn new(image: DynamicImage) -> Self {
        Image { image }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let image = image::open(path)?;
        Ok(Image { image })
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.image.width())
            .field("height", &self.image.height())
            .finish()
    }
}

impl From<DynamicImage> for Image {
    fn from(image: DynamicImage) -> Self {
        Image { image }
    }
}

impl HasColorValue for Image {
    fn value_at(&self, coords: TextureCoordinates, _: Point3) -> Color {
        if self.image.height() == 0 {
            return Vec3::new(0.0, 1.0, 1.0);
        }
        let mut coords = coords.clamp01();
        // Flip V to image coordinates
        coords.v = 1.0 - coords.v;

        // TODO anti-aliasing
        let i = ((coords.u * self.image.width() as f32) as u32).min(self.image.height() - 1);
        let j = ((coords.v * self.image.height() as f32) as u32).min(self.image.height() - 1);
        let pixel = self.image.get_pixel(i, j);
        let color_scale = 1.0 / 255.0;

        Vec3::new(
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

impl Texture {
    pub fn solid_color(albedo: Color) -> Arc<Self> {
        Arc::new(Texture::SolidColor(SolidColor { albedo }))
    }

    pub fn image(image: DynamicImage) -> Arc<Self> {
        Arc::new(Texture::Image(Image::new(image)))
    }
}
