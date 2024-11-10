use std::cmp;

use ratatui::layout::{Constraint, Layout, Rect};
use text::width;

use crate::{
    buffer::{Buffer, BufferId},
    cursor::Cursor,
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

        self.reorder_window_size();

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

    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Window> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        self.nodes.iter_mut()
    }

    pub fn reorder_window_size(&mut self) {
        let window_count = self.count() as u16;
        let constraints: Vec<Constraint> = (0..window_count)
            .map(|_| Constraint::Percentage(100 / window_count))
            .collect();

        let window_layout = Layout::horizontal(constraints).split(self.area);

        for (i, window) in &mut self.nodes.iter_mut().enumerate() {
            let area = window_layout[i];
            window.set_area(area);
        }
    }
}

#[derive(Default, Copy, Clone)]
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

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
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
            width(&line_to_cursor)
        };

        let screen_width = (self.area.width as usize).saturating_sub(1);
        let cur_line_width = width(&content.line(y));

        self.offset.horizontal = cmp::min(self.offset.horizontal, x.saturating_sub(scrolloff));
        if x >= self.offset.horizontal + screen_width.saturating_sub(scrolloff)
            && cur_line_width != (self.offset.horizontal + screen_width)
        {
            self.offset.horizontal = x.saturating_sub(screen_width.saturating_sub(scrolloff)) + 1;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub usize);
