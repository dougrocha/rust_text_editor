use color_eyre::eyre::Result;
use ratatui::layout::{Position, Rect};

use crate::tui;

#[derive(Default)]
pub struct Window {
    id: usize,
    buffer_id: usize,

    offset: Position,
    cursor: Position,
}

impl Window {
    pub fn new(id: usize, buffer_id: usize) -> Self {
        Self {
            id,
            buffer_id,
            ..Default::default()
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_buffer_id(&self) -> usize {
        self.buffer_id
    }

    pub fn draw(&mut self, f: &mut tui::Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
}
