use std::cmp;

use ropey::{iter::Chunks, str_utils::byte_to_char_idx, RopeSlice};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
use unicode_width::UnicodeWidthStr;

use crate::{cursor::Cursor, editor::Editor};

// eventually keep track of multiple keypress and if a number was inserted before
pub struct Context<'a> {
    pub editor: &'a mut Editor,
}

/// Returns the display length of a unicode string
pub fn width(content: &RopeSlice) -> usize {
    // as_str will sometimes panic while crossing chunk borders
    match content.as_str() {
        Some(str) => UnicodeWidthStr::width(str),
        None => {
            let content: String = content.chars().collect();
            UnicodeWidthStr::width(content.as_str())
        }
    }
}

pub fn move_right_nth(context: &mut Context, count: usize) {
    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();
    let cursor = buf.get_cursor(focused_window.id);
    let content = &buf.content().slice(..);

    let new_start = next_grapheme_boundary_nth(content, cursor.range.start, count);
    let cursor = Cursor {
        range: new_start..next_grapheme_boundary(content, new_start),
    };

    buf.set_cursor(focused_window.id, cursor);
}

#[inline]
pub fn move_right(context: &mut Context) {
    move_right_nth(context, 1);
}

#[inline]
pub fn move_up(context: &mut Context) {
    let n = 1;

    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();
    let cursor = buf.get_cursor(focused_window.id);
    let content = &buf.content().slice(..);

    let cur_line_index = content.char_to_line(cursor.range.start);
    let new_line_index = cmp::max(0, cur_line_index.saturating_sub(n));

    let cur_col = {
        let cur_line_index = content.line_to_char(cur_line_index);
        let line_to_cursor = content.slice(cur_line_index..cursor.range.start);
        width(&line_to_cursor)
    };

    let new_line = content.line(new_line_index);
    let new_line_width = width(&new_line);
    let char_offset = content.line_to_char(new_line_index) + cmp::min(new_line_width, cur_col);

    let cursor = Cursor {
        range: char_offset..next_grapheme_boundary(content, char_offset),
    };

    buf.set_cursor(focused_window.id, cursor);
}

#[inline]
pub fn move_down(context: &mut Context) {
    let n = 1;

    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();
    let cursor = buf.get_cursor(focused_window.id);
    let content = &buf.content().slice(..);

    let lines_len = content.len_lines().saturating_sub(1);

    let cur_line_index = content.char_to_line(cursor.range.start);
    let new_line_index = cmp::min(cur_line_index + n, lines_len);

    let cur_col = {
        let cur_line_index = content.line_to_char(cur_line_index);
        let line_to_cursor = content.slice(cur_line_index..cursor.range.start);
        width(&line_to_cursor)
    };

    let new_line = content.line(new_line_index);
    let new_line_width = width(&new_line);
    let char_offset = content.line_to_char(new_line_index) + cmp::min(new_line_width, cur_col);

    let cursor = Cursor {
        range: char_offset..next_grapheme_boundary(content, char_offset),
    };

    buf.set_cursor(focused_window.id, cursor);
}

#[inline]
pub fn move_left(context: &mut Context) {
    let n = 1;

    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();
    let cursor = buf.get_cursor(focused_window.id);
    let content = &buf.content().slice(..);

    let new_start = prev_grapheme_boundary_nth(content, cursor.range.start, n);
    let cursor = Cursor {
        range: new_start..next_grapheme_boundary(content, new_start),
    };

    buf.set_cursor(focused_window.id, cursor);
}

#[inline]
pub fn insert_char(context: &mut Context, char: char) {
    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();
    let cursor_pos = buf.get_cursor(focused_window.id).range.start;

    let content = buf.content_mut();
    content.insert_char(cursor_pos, char);

    move_right(context);
}

#[inline]
pub fn insert_new_line(context: &mut Context) {
    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();
    let cursor_pos = buf.get_cursor(focused_window.id).range.start;

    let content = buf.content_mut();
    content.insert_char(cursor_pos, '\n');

    move_down(context);
    goto_start_of_line(context);
}

#[inline]
pub fn delete_char(context: &mut Context) {
    move_left(context);

    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();
    let cursor_pos = buf.get_cursor(focused_window.id).range.start;

    buf.content_mut().remove(cursor_pos..cursor_pos + 1);
}

#[inline]
pub fn goto_start_of_line(context: &mut Context) {
    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();

    let content = &buf.content().slice(..);
    let cursor_pos = buf.get_cursor(focused_window.id).range.start;

    let line_index = content.char_to_line(cursor_pos);
    let start_index = content.line_to_char(line_index);

    let cursor = Cursor {
        range: start_index..next_grapheme_boundary(content, start_index),
    };

    buf.set_cursor(focused_window.id, cursor)
}

pub fn goto_end_of_line(context: &mut Context) {
    let focused_window = context.editor.windows.get_focused().unwrap();
    let buf = context
        .editor
        .buffers
        .get_mut(focused_window.buffer_id)
        .unwrap();

    let content = &buf.content().slice(..);
    let cursor_pos = buf.get_cursor(focused_window.id).range.start;

    let line_index = content.char_to_line(cursor_pos);
    if width(&buf.content().line(line_index)) == 0 {
        return;
    }

    let start_index = content.line_to_char(line_index + 1);
    let index = prev_grapheme_boundary(content, start_index);

    let cursor = Cursor {
        range: index..next_grapheme_boundary(content, index),
    };

    buf.set_cursor(focused_window.id, cursor)
}

/// An implementation of a graphemes iterator, for iterating over
/// the graphemes of a RopeSlice.
pub struct RopeGraphemes<'a> {
    text: RopeSlice<'a>,
    chunks: Chunks<'a>,
    cur_chunk: &'a str,
    cur_chunk_start: usize,
    cursor: GraphemeCursor,
}

impl<'a> RopeGraphemes<'a> {
    pub fn new<'b>(slice: &RopeSlice<'b>) -> RopeGraphemes<'b> {
        let mut chunks = slice.chunks();
        let first_chunk = chunks.next().unwrap_or("");
        RopeGraphemes {
            text: *slice,
            chunks,
            cur_chunk: first_chunk,
            cur_chunk_start: 0,
            cursor: GraphemeCursor::new(0, slice.len_bytes(), true),
        }
    }
}

impl<'a> Iterator for RopeGraphemes<'a> {
    type Item = RopeSlice<'a>;

    fn next(&mut self) -> Option<RopeSlice<'a>> {
        let a = self.cursor.cur_cursor();
        let b;
        loop {
            match self
                .cursor
                .next_boundary(self.cur_chunk, self.cur_chunk_start)
            {
                Ok(None) => {
                    return None;
                }
                Ok(Some(n)) => {
                    b = n;
                    break;
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    self.cur_chunk_start += self.cur_chunk.len();
                    self.cur_chunk = self.chunks.next().unwrap_or("");
                }
                Err(GraphemeIncomplete::PreContext(idx)) => {
                    let (chunk, byte_idx, _, _) = self.text.chunk_at_byte(idx.saturating_sub(1));
                    self.cursor.provide_context(chunk, byte_idx);
                }
                _ => unreachable!(),
            }
        }

        if a < self.cur_chunk_start {
            let a_char = self.text.byte_to_char(a);
            let b_char = self.text.byte_to_char(b);

            Some(self.text.slice(a_char..b_char))
        } else {
            let a2 = a - self.cur_chunk_start;
            let b2 = b - self.cur_chunk_start;
            Some((&self.cur_chunk[a2..b2]).into())
        }
    }
}

/// Finds the next grapheme boundary after the given char position + count.
pub fn next_grapheme_boundary_nth(slice: &RopeSlice, char_idx: usize, count: usize) -> usize {
    // Bounds check
    debug_assert!(char_idx <= slice.len_chars());

    // We work with bytes for this, so convert.
    let mut byte_idx = slice.char_to_byte(char_idx);

    // Get the chunk with our byte index in it.
    let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = slice.chunk_at_byte(byte_idx);

    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);

    // Find the next grapheme cluster boundary.
    for _ in 0..count {
        loop {
            match gc.next_boundary(chunk, chunk_byte_idx) {
                Ok(None) => return slice.len_chars(),
                Ok(Some(n)) => {
                    byte_idx = n;
                    break;
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    chunk_byte_idx += chunk.len();
                    let (a, _, c, _) = slice.chunk_at_byte(chunk_byte_idx);
                    chunk = a;
                    chunk_char_idx = c;
                }
                Err(GraphemeIncomplete::PreContext(n)) => {
                    let ctx_chunk = slice.chunk_at_byte(n - 1).0;
                    gc.provide_context(ctx_chunk, n - ctx_chunk.len());
                }
                _ => unreachable!(),
            }
        }
    }

    let tmp = byte_to_char_idx(chunk, byte_idx - chunk_byte_idx);
    chunk_char_idx + tmp
}

/// Finds the previous grapheme boundary before the given char position + count.
pub fn prev_grapheme_boundary_nth(slice: &RopeSlice, char_idx: usize, count: usize) -> usize {
    // Bounds check
    debug_assert!(char_idx <= slice.len_chars());

    // We work with bytes for this, so convert.
    let mut byte_idx = slice.char_to_byte(char_idx);

    // Get the chunk with our byte index in it.
    let (mut chunk, mut chunk_byte_idx, mut chunk_char_idx, _) = slice.chunk_at_byte(byte_idx);

    // Set up the grapheme cursor.
    let mut gc = GraphemeCursor::new(byte_idx, slice.len_bytes(), true);

    // Find the previous grapheme cluster boundary.
    for _ in 0..count {
        loop {
            match gc.prev_boundary(chunk, chunk_byte_idx) {
                Ok(None) => return 0,
                Ok(Some(n)) => {
                    byte_idx = n;
                    break;
                }
                Err(GraphemeIncomplete::PrevChunk) => {
                    let (a, b, c, _) = slice.chunk_at_byte(chunk_byte_idx - 1);
                    chunk = a;
                    chunk_byte_idx = b;
                    chunk_char_idx = c;
                }
                Err(GraphemeIncomplete::PreContext(n)) => {
                    let ctx_chunk = slice.chunk_at_byte(n - 1).0;
                    gc.provide_context(ctx_chunk, n - ctx_chunk.len());
                }
                _ => unreachable!(),
            }
        }
    }

    let tmp = byte_to_char_idx(chunk, byte_idx - chunk_byte_idx);
    chunk_char_idx + tmp
}

/// Finds the next grapheme boundary after the given char position.
pub fn next_grapheme_boundary(slice: &RopeSlice, char_idx: usize) -> usize {
    next_grapheme_boundary_nth(slice, char_idx, 1)
}

/// Finds the previous grapheme boundary before the given char position.
pub fn prev_grapheme_boundary(slice: &RopeSlice, char_idx: usize) -> usize {
    prev_grapheme_boundary_nth(slice, char_idx, 1)
}
