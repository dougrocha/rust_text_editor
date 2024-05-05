use std::{cmp, ops::Range};

use ropey::Rope;
use unicode_width::UnicodeWidthStr;

use crate::text::{
    next_grapheme_boundary, next_grapheme_boundary_nth, prev_grapheme_boundary_nth, text_width,
};

#[derive(Default)]
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
        let new_start = next_grapheme_boundary_nth(&content.slice(..), self.range.start, n);
        self.range = new_start..next_grapheme_boundary(&content.slice(..), new_start);
    }

    #[inline]
    pub fn move_left(&mut self, content: &Rope, n: usize) {
        let new_start = prev_grapheme_boundary_nth(&content.slice(..), self.range.start, n);
        self.range = new_start..next_grapheme_boundary(&content.slice(..), new_start);
    }

    #[inline]
    pub fn insert_char(&mut self, content: &mut Rope, char: char) {
        content.insert_char(self.range.start, char);
        self.move_right(content, 1);
    }

    #[inline]
    pub fn insert_newline(&mut self, content: &mut Rope) {
        content.insert_char(self.range.start, '\n');

        self.move_down(content, 1);
        self.move_to_start_of_line(content);
    }

    /// Move to end of line
    #[inline]
    pub fn move_to_end_of_line(&mut self, content: &Rope) {
        let cur_line_index = content.char_to_line(self.range.start);
        let cur_line = content.line(cur_line_index);
        let line_width = text_width(&cur_line);

        let start_index = content.line_to_char(cur_line_index);

        let end = start_index + line_width;
        let start = cmp::max(start_index, end.saturating_sub(1));

        self.range = start..end;
    }

    /// Move to start of line
    #[inline]
    pub fn move_to_start_of_line(&mut self, content: &Rope) {
        let cur_line_index = content.char_to_line(self.range.start);
        let start_index = content.line_to_char(cur_line_index);

        self.range = start_index..start_index + 1;
    }

    /// Move to end of buffer
    #[inline]
    pub fn move_to_end_of_buffer(&mut self, content: &Rope) {
        let buf_len = content.len_chars();

        self.range = buf_len..buf_len;
    }

    /// Move to start of buffer
    #[inline]
    pub fn move_to_start_of_buffer(&mut self, content: &Rope) {
        self.range = 0..next_grapheme_boundary(&content.slice(..), 0);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn new_line_works() {
        let mut cursor = Cursor::with_range(5, 6);
        let mut content = Rope::from_str("Hello!\nWorld!");

        assert_eq!(content.len_lines(), 2);

        cursor.insert_newline(&mut content);

        assert_eq!(content.len_lines(), 3);
        assert_eq!(content.to_string(), "Hello\n!\nWorld!");
    }

    #[test]
    fn insert_char_works() {
        let mut cursor = Cursor::with_range(4, 5);
        let mut content = Rope::from_str("Hell!\nWorld!");

        cursor.insert_char(&mut content, 'o');

        assert_eq!(content.to_string(), "Hello!\nWorld!");
    }
}
