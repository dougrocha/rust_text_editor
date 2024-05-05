use ropey::{Rope, RopeSlice};
use std::path::PathBuf;

use color_eyre::eyre::Result;
use ratatui::layout::Rect;

use crate::{
    action::{BufferAction, BuffersAction, CursorAction},
    cursor::Cursor,
    editor::Context,
    text::{text_width, RopeGraphemes},
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
        let cursor = &mut self.cursors[cursor_id.0];

        let content = &self.content;

        match action {
            CursorAction::Up(n) => {
                cursor.move_up(content, n);
            }
            CursorAction::Right(n) => {
                cursor.move_right(content, n);
            }
            CursorAction::Left(n) => {
                cursor.move_left(content, n);
            }
            CursorAction::Down(n) => {
                cursor.move_down(content, n);
            }
            CursorAction::EndOfLine => cursor.move_to_end_of_line(content),
            CursorAction::StartOfLine => cursor.move_to_start_of_line(content),
            _ => {}
        }

        // Mutable actions
        let content = &mut self.content;

        match action {
            CursorAction::InsertChar(character) => {
                cursor.insert_char(content, character);
            }
            CursorAction::InsertNewLine => cursor.insert_newline(content),
            _ => {}
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

        self.draw_lines(f, buffer_layout[0], cursor_id, context)?;
        self.draw_status_line(f, buffer_layout[1], cursor_id, context)?;

        Ok(())
    }

    fn draw_lines(
        &self,
        f: &mut Frame<'_>,
        area: Rect,
        cursor_id: CursorId,
        context: &Context,
    ) -> Result<()> {
        use ratatui::prelude::*;

        let text = self
            .content
            .lines()
            .map(|line| line.to_string())
            .collect::<String>();

        f.render_widget(Text::from(text), area);

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

        let line_index = self.content.char_to_line(cursor.range.start);

        let cur_col = text_width(
            &self
                .content
                .slice(self.content.line_to_char(line_index)..cursor.range.start),
        );

        let cursor_pos = Span::styled(
            format!(
                " {}|{} {}|{} ",
                line_index + 1,
                self.content.len_lines(),
                cur_col + 1,
                text_width(&self.content.line(line_index)),
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
