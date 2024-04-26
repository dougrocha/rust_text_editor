use std::{fs, path::PathBuf};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct Editor {
    config: Config,
    command_tx: Option<UnboundedSender<Action>>,
    buffer: Vec<String>,
}

impl Editor {
    pub fn new(file_paths: Option<Vec<PathBuf>>) -> Self {
        let buffer = fs::read_to_string(file_paths.unwrap().first().unwrap()).unwrap();
        let buffer = buffer.lines().map(|line| line.into()).collect();

        Self {
            buffer,
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

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // TODO: Make a seperate buffer struct to keep track of individual buffers
        // Use the Span to setup text on the frame
        let lines: Vec<Line> = self
            .buffer
            .iter()
            .map(|line| Line::from(line.to_owned()))
            .collect();

        let line = Text::from(lines);

        f.render_widget(line, area);
        Ok(())
    }
}
