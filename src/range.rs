#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub min: f32,
    pub max: f32,
}

impl Range {
    pub const EMPTY: Range = Range::new(f32::NEG_INFINITY, f32::INFINITY);
    pub const UNIVERSE: Range = Range::new(f32::NEG_INFINITY, f32::INFINITY);

    pub const fn new(min: f32, max: f32) -> Self {
        Range { min, max }
    }

    pub fn from_ranges(a: Range, b: Range) -> Self {
        Range {
            min: if a.min <= b.min { a.min } else { b.min },
            max: if a.max >= b.max { a.max } else { b.max },
        }
    }

    pub fn contains(&self, value: f32) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn surrounds(&self, value: f32) -> bool {
        self.min < value && value < self.max
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn clamp(&self, x: f32) -> f32 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }

    pub fn expand(&self, delta: f32) -> Self {
        let padding = delta / 2.0;
        Range {
            min: self.min - padding,
            max: self.max + padding,
        }
    }
}
