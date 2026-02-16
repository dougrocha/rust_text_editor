use std::cmp;

use ratatui::layout::Rect;
use text::width;

use crate::buffer::{Buffer, BufferId};

#[derive(Default, Debug, Copy, Clone)]
pub struct Offset {
    pub vertical: usize,
    pub horizontal: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub id: WindowId,
    pub buffer_id: BufferId,
    pub focused: bool,
    /// y-offset only
    pub offset: Offset,
    /// area for single window
    pub area: Rect,
}

impl Window {
    pub fn new(id: WindowId, buffer_id: BufferId, area: Rect) -> Self {
        Self {
            id,
            buffer_id,
            focused: false,
            offset: Offset::default(),
            area,
        }
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    pub fn position_cursor_in_view(&mut self, buf: &Buffer, scrolloff: usize) {
        let height = (self.area.height as usize).saturating_sub(1);

        let content = buf.content().slice(..);
        let cursor = buf.get_cursor(&self.id).range.start;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub usize);
