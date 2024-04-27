use std::{fs, path::PathBuf};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Position, prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action, buffer::Buffers, components::Action::Buffer, config::Config, mode::Mode,
    utils::version, window::Windows,
};

#[derive(Default)]
pub struct Editor {
    config: Config,
    command_tx: Option<UnboundedSender<Action>>,

    buffers: Buffers,
    windows: Windows,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Editor {
    fn init(&mut self, context: crate::editor::Context, area: Rect) -> Result<()> {
        for file_path in context.file_paths {
            let buffer_id = self.buffers.add(file_path);
            self.windows.add(buffer_id);
        }

        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(keymap) = self.config.keybindings.get(&Mode::Normal) {}

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Buffer(action) => {
                self.buffers.handle_events(action)?;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if self.windows.is_empty() {
            let rects = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(1), // first row
                    Constraint::Min(0),
                ])
                .split(area);
            let rect = rects[0];

            let block = Block::default()
                .title(block::Title::from(version().dim()).alignment(Alignment::Center));
            f.render_widget(block, rect);
        } else {
            for window in &self.windows.nodes {
                let buffer_id = window.buffer_id;
                let buffer = self.buffers.get(buffer_id);
                if let Some(buffer) = buffer {
                    let lines: Vec<Line> = buffer
                        .content
                        .iter()
                        .map(|line| Line::from(line.clone()))
                        .collect();

                    f.render_widget(Text::from(lines), area);

                    f.set_cursor(window.cursor.x, window.cursor.y);
                }
            }
        }

        Ok(())
    }
}
