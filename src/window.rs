use ratatui::layout::Rect;

use crate::{buffer::BufferId, cursor::Cursor};

#[derive(Default)]
pub struct Windows {
    pub nodes: Vec<Window>,
    pub focused_node: Option<WindowId>,

    /// total area for windows
    area: Rect,
}

impl Windows {
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            ..Default::default()
        }
    }

    pub fn add(&mut self, buffer_id: BufferId) -> WindowId {
        // when adding multiple windows, eventually handle different sizes with layout
        let id = WindowId(self.nodes.len());
        self.nodes.push(Window::new(id, buffer_id, self.area));

        id
    }

    pub fn focus(&mut self, id: WindowId) {
        for node in self.nodes.iter_mut() {
            if node.id == id {
                self.focused_node = Some(id);
                node.focused = true;
            }
        }
    }

    pub fn get_by_buffer_id(&self, buffer_id: BufferId) -> Option<&Window> {
        self.nodes.iter().find(|node| node.buffer_id == buffer_id)
    }

    pub fn get_focused(&self) -> Option<&Window> {
        if let Some(focused_node) = self.focused_node {
            self.nodes.get(focused_node.0)
        } else {
            None
        }
    }

    pub fn get_focused_mut(&mut self) -> Option<&mut Window> {
        if let Some(focused_node) = self.focused_node {
            self.nodes.get_mut(focused_node.0)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

pub struct Window {
    pub id: WindowId,
    pub buffer_id: BufferId,
    pub focused: bool,
    /// y-offset only
    pub offset: usize,
    /// area for single window
    pub area: Rect,
    pub cursor: Cursor,
}

impl Window {
    pub fn new(id: WindowId, buffer_id: BufferId, area: Rect) -> Self {
        Self {
            id,
            buffer_id,
            focused: false,
            offset: 0,
            area,
            cursor: Cursor::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub usize);
