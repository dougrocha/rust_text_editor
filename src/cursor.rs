use std::{cmp, ops::Range};

use ropey::Rope;
use unicode_width::UnicodeWidthStr;

use crate::text::{
    next_grapheme_boundary, next_grapheme_boundary_n, prev_grapheme_boundary_n, text_width,
};

#[derive(Default)]
pub struct Cursor {
    pub range: Range<usize>,
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn screen_position(&self, content: &Rope) -> (u16, u16) {
        let y = content.char_to_line(self.range.start);

        let x = {
            let cur_line_index = content.line_to_char(y);
            let line_to_cursor = content.slice(cur_line_index..self.range.start);
            text_width(&line_to_cursor)
        };

        (x as u16, y as u16)
    }

    // handle a lot of cursor specific things in this impl

    #[inline]
    pub fn move_up(&mut self, content: &Rope, n: usize) {
        let cur_line_index = content.char_to_line(self.range.start);
        let new_line_index = cmp::max(0, cur_line_index.saturating_sub(n));

        let cur_col = {
            let cur_line_index = content.line_to_char(cur_line_index);
            let line_to_cursor = content.slice(cur_line_index..self.range.start);
            UnicodeWidthStr::width(line_to_cursor.as_str().unwrap())
        };

        let new_line = content.line(new_line_index);
        let new_line_width = text_width(&new_line);
        let char_offset = content.line_to_char(new_line_index) + cmp::min(new_line_width, cur_col);

        self.range = char_offset..next_grapheme_boundary(&content.slice(..), char_offset);
    }

    #[inline]
    pub fn move_down(&mut self, content: &Rope, n: usize) {
        let lines_len = content.len_lines().saturating_sub(1);

        let cur_line_index = content.char_to_line(self.range.start);
        let new_line_index = cmp::min(cur_line_index + n, lines_len);

        let cur_col = {
            let cur_line_index = content.line_to_char(cur_line_index);
            let line_to_cursor = content.slice(cur_line_index..self.range.start);
            UnicodeWidthStr::width(line_to_cursor.as_str().unwrap())
        };

        let new_line = content.line(new_line_index);
        let new_line_width = text_width(&new_line);
        let char_offset = content.line_to_char(new_line_index) + cmp::min(new_line_width, cur_col);

        self.range = char_offset..next_grapheme_boundary(&content.slice(..), char_offset);
    }

    #[inline]
    pub fn move_right(&mut self, content: &Rope, n: usize) {
        let new_start = next_grapheme_boundary_n(&content.slice(..), self.range.start, n);
        self.range = new_start..next_grapheme_boundary(&content.slice(..), new_start);
    }

    #[inline]
    pub fn move_left(&mut self, content: &Rope, n: usize) {
        let new_start = prev_grapheme_boundary_n(&content.slice(..), self.range.start, n);
        self.range = new_start..next_grapheme_boundary(&content.slice(..), new_start);
    }

    #[inline]
    pub fn insert_char(&mut self, content: &mut Rope, char: char) {
        content.insert_char(self.range.start, char);
        self.move_right(content, 1);
    }

    /// Move to end of line
    #[inline]
    pub fn move_to_end_of_line(&mut self, content: &Rope) {
        let cur_line_index = content.char_to_line(self.range.start);
        let start_index = content.line_to_char(cur_line_index);

        let cur_line = content.line(cur_line_index);

        let line_width = UnicodeWidthStr::width(cur_line.as_str().unwrap());

        let end = start_index + line_width;
        let start = cmp::max(start_index, end.saturating_sub(1));

        self.range = start..end;
    }
}
