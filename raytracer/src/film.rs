use glam::U64Vec2;

use crate::camera::Bounds2;
use crate::spectrum::DenselySampled;

const N_SWATCH_REFLECTANCES: usize = 24;

pub struct SpectralFilm {
    full_resolution: U64Vec2,
    pixel_bounds: Bounds2, // TODO integer version
}

impl SpectralFilm {}

pub struct PixelSensor {
    r_bar: DenselySampled,
    g_bar: DenselySampled,
    b_bar: DenselySampled,
    imaging_ratio: f32,
}

impl PixelSensor {}
