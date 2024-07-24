pub mod aabb;
pub mod camera;
pub mod color;
pub mod film;
pub mod filter;
pub mod integrator;
pub mod macros;
pub mod math;
pub mod random;
pub mod range;
pub mod ray;
pub mod sample;
pub mod spectrum;
pub mod util;
pub mod vec;

pub type Result<T> = color_eyre::Result<T>;
