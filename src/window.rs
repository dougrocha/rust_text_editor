use std::{cell::RefCell, cmp, rc::Rc};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::{action::Action, buffer::Buffer, component::Component, frame::Frame, Rect};

pub struct Window {
    pub id: i32,

    pub size: Rect,
    pub buffer: Rc<RefCell<Buffer>>,

    pub cursor: (u16, u16),
    pub offset: (usize, usize),

    pub focused: bool,
}

impl Window {
    pub fn new(id: i32, size: Rect, buffer: Rc<RefCell<Buffer>>) -> Self {
        Self {
            id,

            cursor: (0, 0),
            offset: (0, 0),

            size,
            buffer,
            focused: false,
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn get_cursor(&self) -> (u16, u16) {
        self.cursor
    }

    pub fn get_cursor_frame(&self) -> (u16, u16) {
        let (x, y) = self.cursor;
        let top = self.size.min.y;
        let left = self.size.min.x;
        let (offset_x, offset_y) = self.offset;

        (
            x - offset_x as u16 + left as u16,
            y - offset_y as u16 + top as u16,
        )
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn scroll(&mut self) {
        let scroll_off = 8;

        let cursor_x = self.cursor.0 as usize;
        let cursor_y = self.cursor.1 as usize;

        let width = self.size.width();
        let height = self.size.height();

        let buffer = self.buffer.borrow();
        let buffer_len = buffer.content.len();

        let line_len = buffer.get_line(cursor_y).unwrap().render.len();

        self.offset.1 = cmp::min(self.offset.1, cursor_y.saturating_sub(scroll_off));
        if cursor_y >= self.offset.1 + height.saturating_sub(scroll_off)
            && buffer_len != (self.offset.1 + height)
        {
            self.offset.1 = cursor_y.saturating_sub(height.saturating_sub(scroll_off)) + 1;
        }

        self.offset.0 = cmp::min(self.offset.0, cursor_x.saturating_sub(scroll_off));
        if cursor_x >= self.offset.0 + width.saturating_sub(scroll_off)
            && line_len != (self.offset.0 + width)
        {
            self.offset.0 = cursor_x.saturating_sub(width.saturating_sub(scroll_off)) + 1;
        }
    }

    pub fn move_cursor(&mut self, key_event: KeyEvent) {
        let buffer = self.buffer.borrow();
        let cur_line = buffer.get_line(self.cursor.1 as usize);

        if cur_line.is_none() {
            return;
        }

        let cur_line_len = cur_line.unwrap().render.len() as u16;

        match key_event.code {
            KeyCode::Down => {
                if self.cursor.1 < buffer.content.len() as u16 - 1 {
                    self.cursor.1 += 1;
                }
            }
            KeyCode::Up => {
                self.cursor.1 = self.cursor.1.saturating_sub(1);
            }
            KeyCode::Left => {
                self.cursor.0 = self.cursor.0.saturating_sub(1);
            }
            KeyCode::Right => {
                if self.cursor.0 < cur_line_len {
                    self.cursor.0 += 1;
                }
            }
            _ => {}
        }

        let cur_line = buffer.get_line(self.cursor.1 as usize);
        let cur_line_len = cur_line.unwrap().render.len() as u16;
        if self.cursor.0 > cur_line_len {
            self.cursor.0 = cur_line_len;
        }
    }
}

impl Component for Window {
    fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                self.move_cursor(key_event)
            }
            KeyCode::Char(c) => {
                let mut buffer = self.buffer.borrow_mut();
                self.cursor = buffer.insert_character(c, self.cursor.0, self.cursor.1);
            }

            KeyCode::Backspace => {
                let mut buffer = self.buffer.borrow_mut();
                self.cursor = buffer.delete_character(self.cursor.0, self.cursor.1);
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame) -> Result<()> {
        self.scroll();
        let buffer = self.buffer.borrow();

        let area = f.size;

        let height = area.height();
        let top = area.min.y;
        let left = area.min.x;
        let width = area.width();

        for y in 0..height {
            let line = buffer.get_line(y + self.offset.1);

            for x in 0..width {
                let y_pos = y + top;
                let x_pos = x + left;

                if let Some(line) = line {
                    if let Some(character) = line.render.chars().nth(x + self.offset.0) {
                        f.set_cell(x_pos, y_pos, character.to_string().as_str());
                    } else {
                        f.set_cell(x_pos, y_pos, " ");
                    }
                }
            }
        }

        let cursor = self.get_cursor_frame();
        f.set_cursor_position(cursor.0, cursor.1);

        Ok(())
    }
}
