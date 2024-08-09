#![allow(dead_code, unused_variables, clippy::excessive_precision)]

pub mod bounds;
pub mod camera;
pub mod color;
pub mod film;
pub mod filter;
pub mod integrator;
#[cfg(feature = "wasm")]
pub mod js;
pub mod macros;
pub mod math;
pub mod random;
pub mod range;
pub mod ray;
pub mod sample;
pub mod shape;
pub mod spectrum;
pub mod transform;
pub mod util;
pub mod vec;

pub type Result<T> = color_eyre::Result<T>;
