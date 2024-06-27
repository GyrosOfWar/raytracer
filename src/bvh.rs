use crate::trace::Range;

#[derive(Debug)]
pub struct Aabb {
    pub x: Range,
    pub y: Range,
    pub z: Range,
}
