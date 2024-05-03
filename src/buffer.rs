use ropey::{Rope, RopeSlice};
use std::{cmp, ops::Range, path::PathBuf};

use color_eyre::eyre::Result;
use ratatui::layout::{Position, Rect};

use crate::{
    action::{BufferAction, BuffersAction, CursorAction},
    editor::Context,
    tui::Frame,
    window::CursorId,
};

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

    pub fn add(&mut self, content: Rope, file_path: Option<PathBuf>) -> BufferId {
        let buffer_id = BufferId(self.next_buffer_id);
        self.next_buffer_id += 1;
        self.buffers
            .push(Buffer::new(buffer_id, content, file_path));
        buffer_id
    }

    pub fn find_by_file_path(&self, file_path: &PathBuf) -> Option<BufferId> {
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

    pub fn handle_actions(&mut self, action: BuffersAction) {
        match self.get_mut(action.buffer_id) {
            Some(buffer) => {
                buffer.handle_action(action.inner_action);
            }
            None => todo!(),
        }
    }
}

#[derive(Default)]
pub struct Cursor {
    pub range: Range<usize>,
    visual_horizontal_offset: Option<usize>,
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    // handle a lot of cursor specific things in this impl
}

pub struct Buffer {
    pub id: BufferId,
    pub content: Rope,
    pub file_path: Option<PathBuf>,
    pub cursors: Vec<Cursor>,
}

impl Buffer {
    pub fn new(id: BufferId, content: Rope, file_path: Option<PathBuf>) -> Self {
        match file_path {
            Some(file_path) => Self {
                id,
                file_path: Some(file_path.to_path_buf()),
                content,
                cursors: vec![Cursor::default()],
            },
            None => Self {
                id,
                file_path: None,
                content,
                cursors: vec![Cursor::default()],
            },
        }
    }

    pub fn get_line(&self, index: usize) -> Option<RopeSlice> {
        self.content.get_line(index)
    }

    pub fn get_cursor(&self, cursor_id: CursorId) -> &Cursor {
        &self.cursors[cursor_id.0]
    }

    pub fn get_cursor_mut(&mut self, cursor_id: CursorId) -> &mut Cursor {
        &mut self.cursors[cursor_id.0]
    }

    pub fn handle_action(&mut self, action: BufferAction) {
        match action {
            BufferAction::Save => {}
            BufferAction::CursorAction { cursor_id, action } => {
                self.handle_cursor_action(cursor_id, action);
            }
        }
    }

    pub fn handle_cursor_action(&mut self, cursor_id: CursorId, action: CursorAction) {
        let cursor = self.get_cursor(cursor_id);

        let lines_len = self.content.len_lines() as u16;

        match action {
            _ => {} // CursorAction::Up(n) => {
                    //     self.get_cursor_mut(cursor_id).y = cursor.y.saturating_sub(n as u16);
                    // }
                    // CursorAction::Down(n) => {
                    //     // subtract by two to handle end-of-line case
                    //     // makes it more like vim
                    //     self.get_cursor_mut(cursor_id).y =
                    //         cmp::min(lines_len - 2, cursor.y.saturating_add(n as u16));
                    // }
                    // CursorAction::Left(n) => {
                    //     self.get_cursor_mut(cursor_id).x = cursor.x.saturating_sub(n as u16);
                    // }
                    // CursorAction::Right(n) => {
                    //     // change how much is subtracted dependent on mode of editor
                    //     let line_len = self
                    //         .content
                    //         .line(cursor.y.into())
                    //         .len_chars()
                    //         .saturating_sub(3) as u16;
                    //
                    //     self.get_cursor_mut(cursor_id).x = cmp::min(cursor.x + n as u16, line_len);
                    // }
                    // CursorAction::InsertChar(character) => {
                    //     self.content.insert_char(cursor.x.into(), character);
                    // }
        }
    }

    pub fn draw(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        cursor_id: CursorId,
        context: &Context,
    ) -> Result<()> {
        use ratatui::prelude::*;

        let buffer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Length(1)])
            .split(area);

        let lines: Vec<Line> = self
            .content
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();

        f.render_widget(Text::from(lines), buffer_layout[0]);
        self.draw_status_line(f, buffer_layout[1], cursor_id, context)?;

        Ok(())
    }

    pub fn draw_status_line(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        cursor_id: CursorId,
        context: &Context,
    ) -> Result<()> {
        use ratatui::prelude::*;
        let cursor = self.get_cursor(cursor_id);

        let mode = Span::styled(
            format!(" {} ", context.mode),
            Style::default().bg(Color::Blue).fg(Color::Black),
        );

        let file_name = Span::styled(
            format!(
                " {} ",
                self.file_path.as_ref().unwrap().to_str().unwrap_or("None")
            ),
            Style::default().fg(Color::Gray),
        );

        let char_line_start = self
            .content
            .line_to_char(self.content.char_to_line(cursor.range.start));

        let y = self.content.char_to_line(cursor.range.start);

        let cursor_pos = Span::styled(
            format!(
                " {}|{} {}|{} ",
                y + 1,
                self.content.len_lines().saturating_sub(1),
                // make this work on multiple lines
                cursor.range.start,
                self.content.line(y).len_chars().saturating_sub(2),
            ),
            Style::default(),
        );

        let status_line_layout =
            Layout::horizontal(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(area);

        let left_side = Line::from(vec![mode]).left_aligned().bg(Color::DarkGray);

        let right_side = Line::from(vec![file_name, cursor_pos])
            .right_aligned()
            .bg(Color::DarkGray);

        f.render_widget(left_side, status_line_layout[0]);
        f.render_widget(right_side, status_line_layout[1]);

        Ok(())
    }
}
