use std::fmt;

use crate::impl_binary_op;

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn min(&self, b: Vec2) -> Vec2 {
        Vec2::new(self.x.min(b.x), self.y.min(b.y))
    }

    pub fn max(&self, b: Vec2) -> Vec2 {
        Vec2::new(self.x.max(b.x), self.y.max(b.y))
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl_binary_op!(Add : add => (lhs: Vec2, rhs: Vec2) -> Vec2 {
    Vec2::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
    )
});

impl_binary_op!(Sub : sub => (lhs: Vec2, rhs: Vec2) -> Vec2 {
    Vec2::new(
        lhs.x - rhs.x,
        lhs.y - rhs.y,
    )
});

impl_binary_op!(Mul : mul => (lhs: Vec2, rhs: f32) -> Vec2 {
    Vec2::new(
        lhs.x * rhs,
        lhs.y * rhs,
    )
});

impl_binary_op!(Mul : mul => (lhs: f32, rhs: Vec2) -> Vec2 {
    Vec2::new(
        rhs.x * lhs,
        rhs.y * lhs,
    )
});

impl_binary_op!(Div : div => (lhs: Vec2, rhs: f32) -> Vec2 {
    Vec2::new(
        lhs.x / rhs,
        lhs.y / rhs,
    )
});

#[derive(Debug, Copy, Clone)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Point2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn min(&self, b: IVec2) -> IVec2 {
        IVec2::new(self.x.min(b.x), self.y.min(b.y))
    }

    pub fn max(&self, b: IVec2) -> IVec2 {
        IVec2::new(self.x.max(b.x), self.y.max(b.y))
    }

    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

impl_binary_op!(Add : add => (lhs: IVec2, rhs: IVec2) -> IVec2 {
    IVec2::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
    )
});

impl_binary_op!(Sub : sub => (lhs: IVec2, rhs: IVec2) -> IVec2 {
    IVec2::new(
        lhs.x - rhs.x,
        lhs.y - rhs.y,
    )
});

#[derive(Debug, Copy, Clone)]
pub struct UVec2 {
    pub x: u32,
    pub y: u32,
}

impl UVec2 {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl_binary_op!(Add : add => (lhs: UVec2, rhs: UVec2) -> UVec2 {
    UVec2::new(
        lhs.x + rhs.x,
        lhs.y + rhs.y,
    )
});

impl_binary_op!(Sub : sub => (lhs: UVec2, rhs: UVec2) -> UVec2 {
    UVec2::new(
        lhs.x - rhs.x,
        lhs.y - rhs.y,
    )
});
