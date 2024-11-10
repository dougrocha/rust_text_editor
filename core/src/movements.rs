use std::cmp;

use text::{
    next_grapheme_boundary, next_grapheme_boundary_nth, prev_grapheme_boundary,
    prev_grapheme_boundary_nth, width,
};

use crate::{components::Context, cursor::Cursor};

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

    let new_line_width = width(&content.line(new_line_index));
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

    let new_line_width = width(&content.line(new_line_index));
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

#[inline]
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
