use color_eyre::eyre::Result;
use ratatui::layout::{Position, Rect};

use crate::{action::Action, buffer::BufferId, tui};

#[derive(Default)]
pub struct Windows {
    pub nodes: Vec<Window>,
    focused_node: usize,
}

impl Windows {
    pub fn add(&mut self, buffer_id: BufferId) {
        self.nodes.push(Window::new(self.nodes.len(), buffer_id));
    }

    pub fn get_focused(&self) -> Option<&Window> {
        self.nodes.get(self.focused_node)
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[derive(Default)]
pub struct Window {
    pub id: usize,
    pub buffer_id: BufferId,

    pub cursor: Position,
}

impl Window {
    pub fn new(id: usize, buffer_id: BufferId) -> Self {
        Self {
            id,
            buffer_id,
            ..Default::default()
        }
    }

    pub fn move_right(&mut self) {
        self.cursor.x += 1;
    }
}
