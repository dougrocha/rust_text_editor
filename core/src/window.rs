use std::cmp;

use ratatui::layout::Rect;

use crate::{
    buffer::{Buffer, BufferId},
    cursor::Cursor,
    text,
};

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

#[derive(Default)]
pub struct Offset {
    pub vertical: usize,
    pub horizontal: usize,
}

pub struct Window {
    pub id: WindowId,
    pub buffer_id: BufferId,
    pub focused: bool,
    /// y-offset only
    pub offset: Offset,
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
            offset: Offset::default(),
            area,
            cursor: Cursor::default(),
        }
    }

    pub fn position_cursor_in_view(&mut self, buf: &Buffer, scrolloff: usize) {
        let height = (self.area.height as usize).saturating_sub(1);

        let content = buf.content().slice(..);
        let cursor = buf.get_cursor(self.id).range.start;

        let y = content.char_to_line(cursor);

        self.offset.vertical = cmp::min(self.offset.vertical, y.saturating_sub(scrolloff));
        if y >= self.offset.vertical + height.saturating_sub(scrolloff)
            && content.len_lines() != (self.offset.vertical + height)
        {
            self.offset.vertical = y.saturating_sub(height.saturating_sub(scrolloff)) + 1;
        }

        let x = {
            let cur_line_index = content.line_to_char(y);
            let line_to_cursor = content.slice(cur_line_index..cursor);
            text::width(&line_to_cursor)
        };

        let width = (self.area.width as usize).saturating_sub(1);
        let cur_line_width = text::width(&content.line(y));

        self.offset.horizontal = cmp::min(self.offset.horizontal, x.saturating_sub(scrolloff));
        if x >= self.offset.horizontal + width.saturating_sub(scrolloff)
            && cur_line_width != (self.offset.horizontal + width)
        {
            self.offset.horizontal = x.saturating_sub(width.saturating_sub(scrolloff)) + 1;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub usize);
