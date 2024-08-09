mod macros;
mod mat3;
mod mat4;
mod random;
mod vec2;
mod vec3;

pub use mat3::*;
pub use mat4::*;
pub use vec2::*;
pub use vec3::*;

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

pub fn point2(x: f32, y: f32) -> Point2 {
    Point2::new(x, y)
}

pub fn point3(x: f32, y: f32, z: f32) -> Point3 {
    Point3::new(x, y, z)
}

pub fn ivec2(x: i32, y: i32) -> IVec2 {
    IVec2::new(x, y)
}

pub fn uvec2(x: u32, y: u32) -> UVec2 {
    UVec2::new(x, y)
}

#[derive(Debug, Copy, Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub trait VectorLike<const N: usize, T> {
    /// Get the component at the given index. Panics if the index is out of bounds.
    fn component(&self, index: usize) -> T;

    /// Get the data as an array.
    fn data(&self) -> [T; N];

    /// Create a new instance from the given data.
    fn from_data(data: [T; N]) -> Self;
}
