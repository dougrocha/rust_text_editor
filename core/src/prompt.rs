use crate::{
    components::{Component, Context, EventPropagation, Position},
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
                match self.input.as_str() {
                    "q" => event_context.editor.should_quit = true,

                    "vsplit" | "split" => {
                        let focused_win = event_context.editor.windows.get_focused();
                        let Some(focused_win) = focused_win else {
                            return EventPropagation::Consume(None);
                        };

                        let buf_id = focused_win.buffer_id;
                        let cursor = event_context
                            .editor
                            .buffers
                            .get(buf_id)
                            .unwrap()
                            .get_cursor(&focused_win.id)
                            .clone();

                        let direction = if self.input.chars().nth(0).unwrap() == 'v' {
                            Direction::Vertical
                        } else {
                            Direction::Horizontal
                        };

                        let window_id =
                            event_context
                                .editor
                                .windows
                                .split(focused_win.id, buf_id, direction);

                        event_context
                            .editor
                            .buffers
                            .get_mut(buf_id)
                            .unwrap()
                            .set_cursor(window_id, cursor);
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

                    _ => todo!(),
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
            y: area.height as usize - 1,
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
        let y = area.height - 1;
        let style = Style::default();

        buf.set_string(0, y, ":", style);

        let mut x = 1;
        for char in self.text.chars() {
            buf.set_string(x, y, char.to_string(), style);

            x += 1;
        }
    }
}
