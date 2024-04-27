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

pub struct Editor {
    config: Config,
    command_tx: Option<UnboundedSender<Action>>,

    buffers: Buffers,
    windows: Windows,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            command_tx: None,
            buffers: Buffers::new(),
            windows: Windows::new(),
        }
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
            let version = version();

            let lines: Vec<Line> = version.lines().map(Line::from).collect();

            f.render_widget(Text::from(lines), area);
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

                    f.set_cursor(buffer.cursor.x, buffer.cursor.y);
                }
            }
        }

        Ok(())
    }
}
