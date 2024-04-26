use std::{fs, path::PathBuf};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Position, prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action, buffer::Buffer, config::Config, mode::Mode, utils::version, window::Window,
};

#[derive(Default)]
pub struct Editor {
    config: Config,
    command_tx: Option<UnboundedSender<Action>>,

    buffers: Vec<Buffer>,
    window: Window,
}

impl Editor {
    pub fn new(file_paths: Vec<PathBuf>) -> Self {
        let buffers: Vec<Buffer> = file_paths
            .iter()
            .enumerate()
            .map(|(i, line)| Buffer::new(i, Some(line)))
            .collect();

        let window = Window::new(0, 0);

        Self {
            buffers,
            ..Default::default()
        }
    }
}

impl Component for Editor {
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

        match key.code {
            KeyCode::Char('h' | 'j' | 'k' | 'l') => {
                let some = "";
            }
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let buffer_id = self.window.get_buffer_id();

        let buffer = self.buffers.iter().find(|x| x.id == buffer_id);

        if let Some(buffer) = buffer {
            let lines: Vec<Line> = buffer
                .content
                .iter()
                .map(|line| Line::from(line.clone()))
                .collect();

            f.render_widget(Text::from(lines), area);

            return Ok(());
        }

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

        Ok(())
    }
}
