use super::spectrum::Spectrum;

#[derive(Debug)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Xyz {}

impl From<Spectrum> for Xyz {
    fn from(value: Spectrum) -> Self {
        todo!()
    }
}
