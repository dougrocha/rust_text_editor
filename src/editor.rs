use std::{fs::File, io::BufReader, path::Path};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ropey::Rope;

use crate::{
    buffer::{BufferId, Buffers},
    components::{self, Component, EventPropagation, Position},
    mode::Mode,
    terminal::Event,
    text::{self, width},
    window::Windows,
};

pub struct Editor {
    pub mode: Mode,
    pub needs_redraw: bool,
    pub buffers: Buffers,
    pub windows: Windows,
    should_quit: bool,
}

impl Editor {
    pub fn new(area: Rect) -> Self {
        Self {
            mode: Mode::Normal,
            needs_redraw: false,
            windows: Windows::new(area),
            buffers: Buffers::new(),
            should_quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn open(&mut self, file_path: &Path) -> Result<BufferId> {
        if let Some(buffer_id) = self.buffers.find_by_file_path(file_path) {
            let window_id = self.windows.add(buffer_id);
            self.windows.focus(window_id);

            return Ok(buffer_id);
        }

        let content = Rope::from_reader(BufReader::new(File::open(file_path)?))?;

        let buffer_id = self.buffers.add(content, Some(file_path));
        let window_id = self.windows.add(buffer_id);
        self.windows.focus(window_id);

        Ok(buffer_id)
    }

    fn cursor(&self) -> Option<Position> {
        let focused_window = self.windows.get_focused().unwrap();
        let buf = self.buffers.get(focused_window.buffer_id).unwrap();
        let content = buf.content();

        let cursor = buf.get_cursor(focused_window.id);
        let y = content.char_to_line(cursor.range.start);

        let x = {
            let cur_line_index = content.line_to_char(y);
            let line_to_cursor = content.slice(cur_line_index..cursor.range.start);
            width(&line_to_cursor)
        };

        Some(Position { x, y })
    }
}

#[derive(Default)]
pub struct EditorView {}

impl EditorView {
    pub fn new() -> Self {
        Self {}
    }

    fn handle_key_events(
        &mut self,
        event: &KeyEvent,
        context: &mut components::Context,
    ) -> EventPropagation {
        let mut event_context: text::Context = text::Context {
            editor: context.editor,
        };

        match event.code {
            KeyCode::Char('q') => context.editor.should_quit = true,
            KeyCode::Char('l') => text::move_right(&mut event_context),
            KeyCode::Char('h') => text::move_left(&mut event_context),
            KeyCode::Char('j') => text::move_down(&mut event_context),
            KeyCode::Char('k') => text::move_up(&mut event_context),
            _ => {}
        }

        EventPropagation::Consume(None)
    }
}

impl Component for EditorView {
    fn handle_events(
        &mut self,
        event: &Event,
        context: &mut components::Context,
    ) -> EventPropagation {
        match event {
            Event::Key(key_event) => self.handle_key_events(key_event, context),
            Event::Mouse(_mouse_event) => EventPropagation::Ignore(None),
            _ => EventPropagation::Ignore(None),
        }
    }

    fn cursor(&self, _area: Rect, context: &mut Editor) -> Option<Position> {
        context.cursor()
    }

    fn render(
        &self,
        f: &mut crate::terminal::Frame<'_>,
        area: Rect,
        context: &mut crate::components::Context,
    ) {
        use ratatui::prelude::*;

        // render text lines
        let editor = &context.editor;
        let window = editor.windows.get_focused().unwrap();
        let buf = editor.buffers.get(window.buffer_id).unwrap();

        let lines = buf
            .content()
            .lines()
            .map(|line| line.to_string())
            .collect::<String>();

        let cursor = buf.get_cursor(window.id);

        let line_index = buf.content().char_to_line(cursor.range.start);

        let cur_col = width(
            &buf.content()
                .slice(buf.content().line_to_char(line_index)..cursor.range.start),
        );

        let cursor_pos = Span::styled(
            format!(
                " {}|{} {}|{} ",
                line_index + 1,
                buf.content().len_lines(),
                cur_col + 1,
                width(&buf.content().line(line_index)),
            ),
            Style::default(),
        );

        let buffer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Length(1)])
            .split(area);

        f.render_widget(Text::from(lines), buffer_layout[0]);
        f.render_widget(Text::from(cursor_pos), buffer_layout[1]);
    }
}
