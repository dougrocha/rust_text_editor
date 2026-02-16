use ropey::{Rope, RopeSlice};
use std::{
    collections::HashMap,
    ops::Range,
    path::{Path, PathBuf},
};
use syntax::{byte_to_point, Highlight};
use tree_sitter::{InputEdit, Point};

use crate::{cursor::Cursor, window::WindowId};

pub struct EditInfo {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_point: Point,
    pub old_end_point: Point,
    pub new_end_point: Point,
}

impl EditInfo {
    pub fn to_input_edit(&self) -> InputEdit {
        InputEdit {
            start_byte: self.start_byte,
            old_end_byte: self.old_end_byte,
            new_end_byte: self.new_end_byte,
            start_position: self.start_point,
            old_end_position: self.old_end_point,
            new_end_position: self.new_end_point,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferId(usize);

impl BufferId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Default)]
pub struct Buffers {
    pub buffers: Vec<Buffer>,
    pub next_buffer_id: usize,
}

impl Buffers {
    pub fn new() -> Self {
        Self {
            buffers: vec![],
            next_buffer_id: 0,
        }
    }

    pub fn add(&mut self, content: Rope, file_path: Option<&Path>) -> BufferId {
        let buffer_id = BufferId(self.next_buffer_id);
        self.next_buffer_id += 1;
        self.buffers
            .push(Buffer::new(buffer_id, content, file_path));

        buffer_id
    }

    pub fn find_by_file_path(&self, file_path: &Path) -> Option<BufferId> {
        self.iter()
            .find(|b| {
                if let Some(buf_path) = b.file_path.as_ref() {
                    return buf_path == file_path;
                }
                false
            })
            .map(|b| b.id)
    }

    pub fn get(&self, buffer_id: BufferId) -> Option<&Buffer> {
        self.iter().find(|buf| buf.id == buffer_id)
    }

    pub fn get_mut(&mut self, buffer_id: BufferId) -> Option<&mut Buffer> {
        self.iter_mut().find(|buf| buf.id == buffer_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Buffer> {
        self.buffers.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Buffer> {
        self.buffers.iter_mut()
    }
}

pub struct Buffer {
    pub id: BufferId,
    content: Rope,
    cursors: HashMap<WindowId, Cursor>,
    file_path: Option<PathBuf>,
    pub highlight: Highlight,
}

impl Buffer {
    pub fn new(id: BufferId, content: Rope, file_path: Option<&Path>) -> Self {
        let highlight = Highlight::new(content.slice(..));

        match file_path {
            Some(file_path) => Self {
                id,
                content,
                cursors: HashMap::default(),
                file_path: Some(file_path.to_path_buf()),
                highlight,
            },
            None => Self {
                id,
                content,
                cursors: HashMap::default(),
                file_path: None,
                highlight,
            },
        }
    }

    pub fn content(&self) -> &Rope {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut Rope {
        &mut self.content
    }

    pub fn get_cursor(&self, window_id: &WindowId) -> &Cursor {
        &self.cursors[window_id]
    }

    pub fn set_cursor(&mut self, window_id: WindowId, cursor: Cursor) {
        self.cursors.insert(window_id, cursor);
    }

    pub fn remove_cursor(&mut self, window_id: &WindowId) {
        self.cursors.remove(window_id);
    }

    pub fn get_line(&self, index: usize) -> Option<RopeSlice<'_>> {
        self.content.get_line(index)
    }

    pub fn insert_char(&mut self, pos: usize, char: char) {
        let edit_info = self.build_insert_edit(pos, &char.to_string());

        self.content.insert_char(pos, char);

        self.highlight
            .update(self.content.slice(..), Some(&edit_info.to_input_edit()));
    }

    pub fn insert_str(&mut self, pos: usize, text: &str) {
        let edit_info = self.build_insert_edit(pos, text);

        self.content_mut().insert(pos, text);

        self.highlight
            .update(self.content.slice(..), Some(&edit_info.to_input_edit()));
    }

    pub fn remove(&mut self, range: Range<usize>) {
        let edit_info = self.build_delete_edit(range.start, range.end);

        self.content_mut().remove(range);

        self.highlight
            .update(self.content.slice(..), Some(&edit_info.to_input_edit()));
    }

    pub fn replace_range(&mut self, range: Range<usize>, text: &str) {
        self.content_mut().remove(range.clone());

        let edit_info = self.build_insert_edit(range.start, text);
        self.content_mut().insert(range.start, text);

        self.highlight
            .update(self.content.slice(..), Some(&edit_info.to_input_edit()));
    }

    fn build_insert_edit(&self, char_offset: usize, inserted_text: &str) -> EditInfo {
        let start_byte = self.content.char_to_byte(char_offset);
        let old_end_byte = start_byte;
        let new_end_byte = start_byte + inserted_text.len();

        let start_point = byte_to_point(&self.content.slice(..), start_byte);
        let old_end_point = start_point;

        // For new_end_point, we need to calculate based on inserted text
        let new_end_point = if inserted_text.contains('\n') {
            let lines = inserted_text.lines().count();
            let last_line = inserted_text.lines().last().unwrap_or("");
            Point {
                row: start_point.row + lines - 1,
                column: if lines > 1 {
                    last_line.len()
                } else {
                    start_point.column + last_line.len()
                },
            }
        } else {
            Point {
                row: start_point.row,
                column: start_point.column + inserted_text.len(),
            }
        };

        EditInfo {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_point,
            old_end_point,
            new_end_point,
        }
    }

    /// Build EditInfo for a deletion
    fn build_delete_edit(&self, start_char: usize, end_char: usize) -> EditInfo {
        let start_byte = self.content.char_to_byte(start_char);
        let old_end_byte = self.content.char_to_byte(end_char);
        let new_end_byte = start_byte;

        let start_point = byte_to_point(&self.content.slice(..), start_byte);
        let old_end_point = byte_to_point(&self.content.slice(..), old_end_byte);
        let new_end_point = start_point;

        EditInfo {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_point,
            old_end_point,
            new_end_point,
        }
    }
}
