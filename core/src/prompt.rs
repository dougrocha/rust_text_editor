use crate::{
    components::{Component, Context, EventPropagation, Position},
    mode::Mode,
    terminal::Event,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::Rect, style::Style, widgets::Widget};

#[derive(Default)]
pub struct CommandPrompt {
    input: String,
}

impl CommandPrompt {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_key_events(&mut self, event: &KeyEvent, context: &mut Context) -> EventPropagation {
        let _event_context = Context {
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
                // TODO: take command and do something with it
                todo!();
            }
            KeyCode::Esc => {
                return EventPropagation::Consume(Some(Box::new(|components, context| {
                    context.editor.mode = Mode::Normal;
                    components.pop();
                })));
            }
            _ => {
                tracing::debug!("getting key");
            }
        }

        EventPropagation::Consume(None)
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
