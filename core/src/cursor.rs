use std::ops::Range;

#[derive(Default, Clone)]
pub struct Cursor {
    pub range: Range<usize>,
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_range(start: usize, end: usize) -> Self {
        Self { range: start..end }
    }
}
