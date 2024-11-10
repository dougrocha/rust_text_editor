use crate::{
    components::{Component, Context, EventPropagation},
    terminal::Event,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::Rect;

#[derive(Default)]
pub struct Prompt {}

impl Prompt {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_key_events(&mut self, event: &KeyEvent, context: &mut Context) -> EventPropagation {
        let mut event_context = Context {
            editor: context.editor,
        };

        match event.code {
            KeyCode::Char(char) => todo!(),
            _ => {
                tracing::debug!("getting key");
            }
        }

        EventPropagation::Consume(None)
    }
}

impl Component for Prompt {
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

    fn cursor(
        &self,
        _area: Rect,
        _context: &mut crate::editor::Editor,
    ) -> Option<crate::components::Position> {
        None
    }

    fn render(
        &self,
        f: &mut crate::terminal::Frame<'_>,
        area: Rect,
        context: &mut crate::components::Context,
    ) {
        todo!()
    }
}
