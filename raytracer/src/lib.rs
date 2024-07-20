pub mod aabb;
pub mod color;
pub mod integrator;
pub mod math;
pub mod onb;
pub mod random;
pub mod range;
pub mod ray;
pub mod sample;
pub mod spectrum;
pub mod util;
pub mod vec;

pub type Result<T> = color_eyre::Result<T>;
