use std::{fs, path::PathBuf};

use color_eyre::eyre::Result;
use ratatui::layout::{Position, Rect};
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::action::Action;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuffersAction {
    buffer_id: BufferId,
    inner_action: BufferAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum BufferAction {
    MoveRight,
}

impl From<BuffersAction> for Action {
    fn from(action: BuffersAction) -> Action {
        Action::Buffer(action)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferId(usize);

impl BufferId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

pub struct Buffers {
    pub buffers: Vec<Buffer>,
    pub next_buffer_id: usize,
}

impl Buffers {
    pub fn new() -> Self {
        Self {
            buffers: vec![],
            next_buffer_id: 0,
        }
    }

    pub fn add(&mut self, file_path: PathBuf) -> BufferId {
        let buffer_id = BufferId(self.next_buffer_id);
        self.next_buffer_id += 1;
        self.buffers.push(Buffer::new(buffer_id, Some(file_path)));
        buffer_id
    }

    pub fn get(&self, buffer_id: BufferId) -> Option<&Buffer> {
        self.buffers.iter().find(|buf| buf.id == buffer_id)
    }

    pub fn get_mut(&mut self, buffer_id: BufferId) -> Option<&mut Buffer> {
        self.buffers.iter_mut().find(|buf| buf.id == buffer_id)
    }

    pub fn handle_events(&mut self, action: BuffersAction) -> Result<Option<Action>> {
        match self.get_mut(action.buffer_id) {
            Some(buffer) => {
                buffer.handle_event(action.inner_action);
            }
            None => {}
        }

        Ok(None)
    }
}

pub struct Buffer {
    pub id: BufferId,
    pub content: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub cursor: Position,
}

impl Buffer {
    pub fn new(id: BufferId, file_path: Option<PathBuf>) -> Self {
        match file_path {
            Some(file_path) => {
                let content = fs::read_to_string(&file_path).unwrap();
                let content = content.lines().map(|line| line.to_string()).collect();

                Self {
                    id,
                    file_path: Some(file_path.to_path_buf()),
                    content,
                    cursor: Position::new(0, 0),
                }
            }
            None => Self::default(),
        }
    }

    pub fn get_line(&self, index: usize) -> Option<&String> {
        self.content.get(index)
    }

    pub fn get_line_mut(&mut self, index: usize) -> Option<&mut String> {
        self.content.get_mut(index)
    }

    pub fn handle_event(&mut self, action: BufferAction) {
        match action {
            _ => todo!(),
        }
    }
}
