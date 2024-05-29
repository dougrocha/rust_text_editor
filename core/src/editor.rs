use std::{fs::File, io::BufReader, path::Path};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use ropey::{Rope, RopeSlice};
use syntax::HighlightInfo;
use text::width;

use crate::{
    buffer::{BufferId, Buffers},
    components::{self, Component, EventPropagation, Position},
    cursor::Cursor,
    mode::Mode,
    movements,
    terminal::Event,
    window::{Offset, Windows},
};

pub struct Editor {
    pub mode: Mode,
    pub buffers: Buffers,
    pub windows: Windows,
    should_quit: bool,
}

impl Editor {
    pub fn new(area: Rect) -> Self {
        Self {
            mode: Mode::Normal,
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

        Some(Position {
            x: x - focused_window.offset.horizontal,
            y: y - focused_window.offset.vertical,
        })
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
        let mut event_context: movements::Context = movements::Context {
            editor: context.editor,
        };

        match event_context.editor.mode {
            Mode::Normal => match event.code {
                KeyCode::Char('q') => event_context.editor.should_quit = true,
                KeyCode::Char('l') => movements::move_right(&mut event_context),
                KeyCode::Char('h') => movements::move_left(&mut event_context),
                KeyCode::Char('j') => movements::move_down(&mut event_context),
                KeyCode::Char('k') => movements::move_up(&mut event_context),
                KeyCode::Char('i') => event_context.editor.mode = Mode::Insert,
                KeyCode::Char('0') => movements::goto_start_of_line(&mut event_context),
                KeyCode::Char('$') => movements::goto_end_of_line(&mut event_context),
                //KeyCode::Char('w') => text::move_word_forward(&mut event_context),
                //KeyCode::Char('b') => text::move_word_backward(&mut event_context),
                //KeyCode::Char(num @ ('1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                // user this number to prefix commands
                //}
                KeyCode::Esc => todo!(),
                _ => {
                    tracing::debug!("getting key");
                }
            },
            Mode::Insert => match event.code {
                KeyCode::Char(char) => movements::insert_char(&mut event_context, char),
                KeyCode::Enter => movements::insert_new_line(&mut event_context),
                KeyCode::Backspace => movements::delete_char(&mut event_context),
                KeyCode::Esc => event_context.editor.mode = Mode::Normal,
                _ => {}
            },
            Mode::Visual => todo!(),
            Mode::Search => todo!(),
        }

        let window = event_context.editor.windows.get_focused_mut().unwrap();
        let buf = event_context.editor.buffers.get(window.buffer_id).unwrap();
        window.position_cursor_in_view(buf, 12);

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

        let editor = &context.editor;
        let window = editor.windows.get_focused().unwrap();
        let buf = editor.buffers.get(window.buffer_id).unwrap();

        let content = buf.content().slice(..);

        let cursor = buf.get_cursor(window.id);
        let line_index = content.char_to_line(cursor.range.start);

        let mode = Span::from(editor.mode.to_string());

        let line_info = format!(
            " {:>2}|{:<2} {:>2}|{:<2} ",
            line_index + 1,
            buf.content().len_lines(),
            width(&content.slice(content.line_to_char(line_index)..cursor.range.start)) + 1,
            width(&content.line(line_index)),
        )
        .fg(Color::Black)
        .bg(Color::Rgb(235, 188, 186));

        let buffer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Length(1)])
            .split(area);

        let space = Span::from(format!(
            "{:>w$}",
            "",
            w = buffer_layout[1].width as usize - line_info.width() - mode.width()
        ));

        let status_line = Line::from(vec![mode, space, line_info]).bg(Color::Rgb(31, 29, 46));

        let range = {
            let last_line = content.len_lines().saturating_sub(1);
            let last_visible_line = (window.offset.vertical + window.area.height as usize)
                .saturating_sub(1)
                .min(last_line);

            let start = content.line_to_byte(window.offset.vertical.min(last_line));
            let end = content.line_to_byte(last_visible_line + 1);
            start..end
        };
        let colors = buf.highlight.colors(content.slice(..), range.clone());

        let text = RenderableText {
            content,
            colors,
            cursor,
            offset: window.offset,
        };

        f.render_widget(text, buffer_layout[0]);
        f.render_widget(status_line, buffer_layout[1]);
    }
}

struct RenderableText<'a> {
    content: RopeSlice<'a>,
    colors: Vec<HighlightInfo>,
    cursor: &'a Cursor,
    offset: Offset,
}

impl Widget for RenderableText<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        buf.set_style(area, Style::default().bg(Color::Rgb(25, 23, 36)));

        let content = self.content;
        let start = content.line_to_char(self.offset.vertical);
        let end = if let Ok(char_index) =
            content.try_line_to_char(self.offset.vertical + area.height as usize)
        {
            char_index
        } else {
            content.len_chars()
        };

        let mut x: u16 = 0;
        let mut y: u16 = 0;
        let mut style = Style::default();

        for (index, char) in content.slice(start..end).chars().enumerate() {
            if char == '\n' {
                y += 1;
                x = 0;
                continue;
            }

            if let Some(c) = self
                .colors
                .iter()
                .find(|x| x.range.contains(&(index + start)))
            {
                style = style.fg(c.color);
            } else {
                style = style.fg(Color::White);
            }

            buf.set_string(x, y, char.to_string(), style);

            x += 1;
        }

        //let colors = buf.highlight.colors(content.slice(..), range.clone());
        //
        //let mut text = Text::default();
        //let mut buffer = ratatui::buffer::Buffer {
        //    area,
        //    content: todo!(),
        //};
        //
        //for (index, line) in content
        //    .slice(content.byte_to_line(range.start)..content.byte_to_line(range.end))
        //    .lines()
        //    .enumerate()
        //{
        //    let mut style = Style::default();
        //
        //    let mut render_line = Line::default();
        //
        //    for (index, byte) in line.bytes().enumerate() {
        //        if let Some(color) = colors
        //            .iter()
        //            .find(|x| x.range.start <= index && index > x.range.end)
        //        {
        //            style = style.patch(color.color);
        //        } else {
        //            style = Style::default();
        //        }
        //
        //        render_line.push_span(Span::styled(byte.to_string(), style));
        //    }
        //
        //    text.push_line(render_line);
        //}
    }
}
