use std::{cell::RefCell, cmp, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent};

use crate::{buffer::Buffer, Frame};

#[derive(Clone)]
pub struct Cell {
    symbol: String,
}

impl Cell {
    pub fn set_cell(&mut self, character: &str) {
        self.symbol = character.to_string();
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: " ".to_string(),
        }
    }
}

pub struct Window {
    pub id: i32,

    pub size: Rect,
    pub buffer: Rc<RefCell<Buffer>>,

    cursor: (u16, u16),
    pub offset: (usize, usize),

    focused: bool,
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

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn scroll(&mut self) {
        let scroll_off = 8;

        let cursor_x = self.cursor.0 as usize;
        let cursor_y = self.cursor.1 as usize;

        let width = self.size.width();

        self.offset.1 = cmp::min(self.offset.1, cursor_y.saturating_sub(scroll_off));
        if cursor_y >= self.offset.1 + self.size.height().saturating_sub(scroll_off) {
            self.offset.1 = (self.cursor.1 as usize)
                .saturating_sub(self.size.height().saturating_sub(scroll_off))
                + 1;
        }

        self.offset.0 = cmp::min(self.offset.0, self.cursor.0 as usize);
        if cursor_x >= self.offset.0 + width {
            self.offset.0 = cursor_x - width + 1;
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        let buffer = self.buffer.borrow();
        let cur_line = buffer.get_line(self.cursor.1 as usize);

        if cur_line.is_none() {
            return;
        }

        let cur_line_len = cur_line.unwrap().render.len() as u16;

        if key_event.code == KeyCode::Down {
            if self.cursor.1 < buffer.content.len() as u16 - 1 {
                self.cursor.1 += 1;
            }
        } else if key_event.code == KeyCode::Up {
            self.cursor.1 = self.cursor.1.saturating_sub(1);
        } else if key_event.code == KeyCode::Left {
            self.cursor.0 = self.cursor.0.saturating_sub(1);
        } else if key_event.code == KeyCode::Right {
            if self.cursor.0 < cur_line_len {
                self.cursor.0 += 1;
            }
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let buffer = self.buffer.borrow();

        let height = self.size.height();
        let top = self.size.top();
        let left = self.size.left();
        let width = self.size.width();

        for y in 0..height {
            let line = buffer.get_line(y + self.offset.1);

            for x in 0..width {
                let y_pos = y + top;
                let x_pos = x + left;

                if let Some(line) = line {
                    if let Some(character) = line.render.chars().nth(x + self.offset.0) {
                        frame.set_cell(x_pos, y_pos, character.to_string().as_str());
                    } else {
                        frame.set_cell(x_pos, y_pos, " ");
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2 {
    x: usize,
    y: usize,
}

impl Vec2 {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn width(&self) -> usize {
        self.x
    }

    pub fn height(&self) -> usize {
        self.y
    }
}

#[derive(Debug)]
pub struct Rect {
    /// The start position of the rectangle
    ///
    /// Normally the top-left corner
    pub start: Vec2,

    /// The end position of the rectangle
    ///
    /// Normally the bottom-right corner
    pub end: Vec2,
}

impl Rect {
    pub fn new(start: Vec2, end: Vec2) -> Rect {
        Self { start, end }
    }

    pub fn top(&self) -> usize {
        self.start.y
    }

    pub fn area(&self) -> usize {
        self.width() * self.height()
    }

    pub fn bottom(&self) -> usize {
        self.end.y
    }

    pub fn left(&self) -> usize {
        self.start.x
    }

    pub fn right(&self) -> usize {
        self.end.x
    }

    pub fn width(&self) -> usize {
        self.right() - self.left()
    }

    pub fn height(&self) -> usize {
        self.bottom() - self.top()
    }
}
