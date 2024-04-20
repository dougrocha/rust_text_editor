pub mod action;
pub mod buffer;
pub mod component;
pub mod event;
pub mod frame;
pub mod symbols;
pub mod terminal;
pub mod window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2 {
    pub x: usize,
    pub y: usize,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn width(&self) -> usize {
        self.x
    }

    pub fn height(&self) -> usize {
        self.y
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    /// The start position of the rectangle
    ///
    /// Normally the top-left corner
    pub min: Vec2,

    /// The end position of the rectangle
    ///
    /// Normally the bottom-right corner
    pub max: Vec2,
}

impl Rect {
    pub const ZERO: Self = Self {
        min: Vec2::ZERO,
        max: Vec2::ZERO,
    };

    pub fn new(x0: usize, y0: usize, x1: usize, y1: usize) -> Rect {
        let min = Vec2::new(x0, y0);
        let max = Vec2::new(x1, y1);

        Self { min, max }
    }

    pub fn from_corners(corner0: Vec2, corner1: Vec2) -> Rect {
        Self {
            min: corner0,
            max: corner1,
        }
    }

    pub fn area(&self) -> usize {
        self.width() * self.height()
    }

    pub fn width(&self) -> usize {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> usize {
        self.max.y - self.min.y
    }
}
