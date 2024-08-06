use crate::vec::{IVec2, Vec2};

pub struct Bounds2f {
    p_min: Vec2,
    p_max: Vec2,
}

impl Bounds2f {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Bounds2f {
            p_min: a.min(b),
            p_max: a.max(b),
        }
    }

    pub fn p_min(&self) -> Vec2 {
        self.p_min
    }

    pub fn p_max(&self) -> Vec2 {
        self.p_max
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bounds2i {
    p_min: IVec2,
    p_max: IVec2,
}

impl Bounds2i {
    pub fn new(a: IVec2, b: IVec2) -> Self {
        Bounds2i {
            p_min: a.min(b),
            p_max: a.max(b),
        }
    }

    pub fn p_min(&self) -> IVec2 {
        self.p_min
    }

    pub fn p_max(&self) -> IVec2 {
        self.p_max
    }

    pub fn area(&self) -> i32 {
        let d = self.p_max - self.p_min;
        d.x * d.y
    }

    pub fn x_extent(&self) -> i32 {
        self.p_max.x - self.p_min.x
    }

    pub fn y_extent(&self) -> i32 {
        self.p_max.y - self.p_min.y
    }
}
