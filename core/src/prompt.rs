use std::path::Path;

use crate::{
    components::{Component, Context, EventPropagation, Position},
    editor::Editor,
    mode::Mode,
    terminal::Event,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Direction, prelude::Rect, style::Style, widgets::Widget};

#[derive(Default)]
pub struct CommandPrompt {
    input: String,
}

impl CommandPrompt {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_key_events(&mut self, event: &KeyEvent, context: &mut Context) -> EventPropagation {
        let event_context = Context {
            editor: context.editor,
        };

        match event.code {
            KeyCode::Char(char) => {
                self.input.push(char);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                let mut parts = self.input.split_whitespace();
                let command = parts.next().unwrap_or("");
                let arg = parts.next();

                match command {
                    "q" => {
                        if event_context.editor.has_unsaved_changes() {
                            event_context.editor.set_status_message(
                                "unsaved changes — :w to save or :q! to discard",
                            );
                        } else {
                            event_context.editor.should_quit = true;
                        }
                    }
                    "q!" => event_context.editor.should_quit = true,
                    "w" => {
                        write_buffer(event_context.editor, arg);
                    }
                    "wq" | "x" => {
                        if write_buffer(event_context.editor, arg) {
                            event_context.editor.should_quit = true;
                        }
                    }
                    "wq!" => {
                        write_buffer(event_context.editor, arg);
                        event_context.editor.should_quit = true;
                    }
                    "wqa" | "xa" => {
                        if write_all_buffers(event_context.editor) {
                            event_context.editor.should_quit = true;
                        }
                    }
                    "wqa!" => {
                        write_all_buffers(event_context.editor);
                        event_context.editor.should_quit = true;
                    }

                    "vsplit" | "split" | "hsplit" => {
                        let focused_win = event_context.editor.windows.get_focused();
                        let Some(focused_win) = focused_win else {
                            return EventPropagation::Consume(None);
                        };

                        let old_window_id = focused_win.id;
                        let buf_id = focused_win.buffer_id;
                        let cursor = event_context
                            .editor
                            .buffers
                            .get(buf_id)
                            .unwrap()
                            .get_cursor(&focused_win.id)
                            .clone();

                        let direction = if command.starts_with('v') {
                            Direction::Vertical
                        } else {
                            Direction::Horizontal
                        };

                        let new_window_id =
                            event_context
                                .editor
                                .windows
                                .split(old_window_id, buf_id, direction);

                        event_context
                            .editor
                            .buffers
                            .get_mut(buf_id)
                            .unwrap()
                            .set_cursor(new_window_id, cursor);

                        // Position the cursor in view for BOTH windows
                        // After split, both windows show the same buffer but have different areas
                        let buf = event_context.editor.buffers.get(buf_id).unwrap();

                        // Position cursor for the old (original) window
                        if let Some(old_window) =
                            event_context.editor.windows.get_mut(old_window_id)
                        {
                            old_window.position_cursor_in_view(buf, 12);
                        }

                        // Position cursor for the new window
                        if let Some(new_window) =
                            event_context.editor.windows.get_mut(new_window_id)
                        {
                            new_window.position_cursor_in_view(buf, 12);
                        }
                    }
                    "close" => {
                        let focused_win = event_context.editor.windows.get_focused();
                        let Some(focused_win) = focused_win else {
                            return EventPropagation::Consume(None);
                        };

                        let Some((buf_id, removed_win_id)) =
                            event_context.editor.windows.close(focused_win.id)
                        else {
                            return EventPropagation::Consume(None);
                        };

                        let buf = event_context.editor.buffers.get_mut(buf_id).unwrap();
                        buf.remove_cursor(&removed_win_id);
                    }

                    other => {
                        event_context
                            .editor
                            .set_status_message(format!("unknown command: {other}"));
                    }
                }

                self.input.clear();

                return self.close_prompt();
            }
            KeyCode::Esc => {
                return self.close_prompt();
            }
            _ => {
                tracing::debug!("getting key");
            }
        }

        EventPropagation::Consume(None)
    }

    #[inline]
    /// Close the Command Prompt Window and return to normal mode
    pub fn close_prompt(&self) -> EventPropagation {
        EventPropagation::Consume(Some(Box::new(|components, context| {
            context.editor.mode = Mode::Normal;
            components.pop();
        })))
    }
}

/// Writes the focused buffer to disk (optionally to a new `path`); returns whether it succeeded.
fn write_buffer(editor: &mut Editor, path: Option<&str>) -> bool {
    let Some(window) = editor.windows.get_focused() else {
        editor.set_status_message("no active window");
        return false;
    };
    let buffer_id = window.buffer_id;
    let buffer = editor.buffers.get_mut(buffer_id).unwrap();

    let result = match path {
        Some(path) => buffer.save_as(Path::new(path)),
        None => buffer.save(),
    };

    match result {
        Ok(()) => {
            let name = buffer
                .file_path()
                .map(|path| path.display().to_string())
                .unwrap_or_default();
            editor.set_status_message(format!("\"{name}\" written"));
            true
        }
        Err(err) => {
            editor.set_status_message(format!("write failed: {err}"));
            false
        }
    }
}

/// Writes every modified buffer to disk; returns `true` only if all writes succeeded.
fn write_all_buffers(editor: &mut Editor) -> bool {
    let mut written = 0;
    let mut failures = Vec::new();

    for buffer in editor.buffers.iter_mut() {
        if !buffer.is_modified() {
            continue;
        }
        match buffer.save() {
            Ok(()) => written += 1,
            Err(err) => failures.push(err.to_string()),
        }
    }

    if failures.is_empty() {
        editor.set_status_message(format!("{written} buffer(s) written"));
        true
    } else {
        editor.set_status_message(format!("write failed: {}", failures.join("; ")));
        false
    }
}

impl Component for CommandPrompt {
    fn handle_events(
        &mut self,
        event: &crate::terminal::Event,
        context: &mut crate::components::Context,
    ) -> crate::components::EventPropagation {
        match event {
            Event::Key(key_event) => self.handle_key_events(key_event, context),
            _ => EventPropagation::Ignore(None),
        }
    }

    fn cursor(&self, area: Rect, _context: &mut crate::editor::Editor) -> Option<Position> {
        Some(Position {
            x: self.input.len() + 1,
            y: (area.height as usize).saturating_sub(1),
        })
    }

    fn render(
        &self,
        f: &mut crate::terminal::Frame<'_>,
        area: Rect,
        _context: &mut crate::components::Context,
    ) {
        let prompt_line = PromptLine {
            text: self.input.as_str(),
        };
        f.render_widget(prompt_line, area);
    }
}

struct PromptLine<'a> {
    text: &'a str,
}

impl<'a> Widget for PromptLine<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let y = area.height.saturating_sub(1);
        let style = Style::default();

        buf.set_string(0, y, ":", style);

        let mut x = 1;
        for char in self.text.chars() {
            buf.set_string(x, y, char.to_string(), style);

            x += 1;
        }
    }
}
