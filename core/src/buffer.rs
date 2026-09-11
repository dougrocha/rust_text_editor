use color_eyre::eyre::{eyre, Result};
use ropey::{Rope, RopeSlice};
use std::{
    collections::HashMap,
    fs::File,
    io::BufWriter,
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
    /// Whether the buffer has unsaved edits since it was opened or last written.
    modified: bool,
    pub highlight: Highlight,
}

impl Buffer {
    pub fn new(id: BufferId, content: Rope, file_path: Option<&Path>) -> Self {
        let highlight = Highlight::new(content.slice(..), file_path);

        Self {
            id,
            content,
            cursors: HashMap::default(),
            file_path: file_path.map(Path::to_path_buf),
            modified: false,
            highlight,
        }
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Writes the buffer to its backing file, clearing the modified flag.
    pub fn save(&mut self) -> Result<()> {
        let path = self
            .file_path
            .clone()
            .ok_or_else(|| eyre!("no file name (use :w <path>)"))?;
        self.write_to_path(&path)
    }

    /// Writes the buffer to `path` and adopts it as the backing file.
    pub fn save_as(&mut self, path: &Path) -> Result<()> {
        self.write_to_path(path)?;
        self.file_path = Some(path.to_path_buf());
        Ok(())
    }

    fn write_to_path(&mut self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        self.content.write_to(BufWriter::new(file))?;
        self.modified = false;
        Ok(())
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
        self.modified = true;

        self.highlight
            .update(self.content.slice(..), Some(&edit_info.to_input_edit()));
    }

    pub fn insert_str(&mut self, pos: usize, text: &str) {
        let edit_info = self.build_insert_edit(pos, text);

        self.content_mut().insert(pos, text);
        self.modified = true;

        self.highlight
            .update(self.content.slice(..), Some(&edit_info.to_input_edit()));
    }

    pub fn remove(&mut self, range: Range<usize>) {
        let edit_info = self.build_delete_edit(range.start, range.end);

        self.content_mut().remove(range);
        self.modified = true;

        self.highlight
            .update(self.content.slice(..), Some(&edit_info.to_input_edit()));
    }

    pub fn replace_range(&mut self, range: Range<usize>, text: &str) {
        self.content_mut().remove(range.clone());

        let edit_info = self.build_insert_edit(range.start, text);
        self.content_mut().insert(range.start, text);
        self.modified = true;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(text: &str) -> Buffer {
        Buffer::new(BufferId::new(0), Rope::from_str(text), None)
    }

    fn scratch_rs(text: &str) -> Buffer {
        Buffer::new(
            BufferId::new(0),
            Rope::from_str(text),
            Some(Path::new("scratch.rs")),
        )
    }

    #[test]
    fn new_buffer_is_not_modified() {
        assert!(!scratch("hello").is_modified());
    }

    #[test]
    fn non_rust_buffer_gets_no_highlighting() {
        let mut buf = Buffer::new(
            BufferId::new(0),
            Rope::from_str("let x = 1;\n"),
            Some(Path::new("notes.txt")),
        );
        assert!(buf
            .highlight
            .colors(buf.content().slice(..), 0..buf.content().len_bytes())
            .is_empty());

        buf.insert_str(0, "more text\n");
        assert!(buf
            .highlight
            .colors(buf.content().slice(..), 0..buf.content().len_bytes())
            .is_empty());
    }

    #[test]
    fn highlight_reflects_newly_typed_text() {
        let mut buf = scratch_rs("let x = 1;\n");
        let before = buf.highlight.colors(buf.content().slice(..), 0..buf.content().len_bytes());
        let had_fn_keyword_before = before
            .iter()
            .any(|h| h.color == ratatui::style::Color::Blue);
        assert!(had_fn_keyword_before, "sanity: 'let' should already be a keyword");

        let insert_at = buf.content().len_chars();
        buf.insert_str(insert_at, "fn foo() {}\n");

        let after = buf.highlight.colors(buf.content().slice(..), 0..buf.content().len_bytes());
        let fn_byte_start = buf.content().char_to_byte(insert_at);
        let fn_is_highlighted = after
            .iter()
            .any(|h| h.range.start == fn_byte_start && h.color == ratatui::style::Color::Blue);
        assert!(
            fn_is_highlighted,
            "newly inserted 'fn' keyword should be highlighted after the edit, got: {after:?}"
        );
    }

    #[test]
    fn edits_mark_buffer_modified() {
        let mut buf = scratch("hello");
        buf.insert_char(5, '!');
        assert!(buf.is_modified());
    }

    #[test]
    fn save_without_path_errors() {
        let mut buf = scratch("hello");
        buf.insert_char(5, '!');
        assert!(buf.save().is_err());
        assert!(buf.is_modified(), "failed save must not clear modified flag");
    }

    #[test]
    fn save_as_writes_file_and_clears_modified() {
        let path = std::env::temp_dir().join("rte_save_as_test.txt");
        let _ = std::fs::remove_file(&path);

        let mut buf = scratch("first line\n");
        buf.insert_str(buf.content().len_chars(), "second line\n");
        buf.save_as(&path).unwrap();

        assert!(!buf.is_modified());
        assert_eq!(buf.file_path(), Some(path.as_path()));
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "first line\nsecond line\n"
        );

        // A later `:w` reuses the adopted path.
        buf.insert_str(buf.content().len_chars(), "third line\n");
        assert!(buf.is_modified());
        buf.save().unwrap();
        assert!(!buf.is_modified());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "first line\nsecond line\nthird line\n"
        );

        std::fs::remove_file(&path).unwrap();
    }
}
