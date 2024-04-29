use std::{fs, path::PathBuf};

use color_eyre::eyre::Result;
use ratatui::layout::{Position, Rect};
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{action::Action, window::CursorId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuffersAction {
    pub buffer_id: BufferId,
    pub inner_action: BufferAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum BufferAction {
    Save,
    CursorAction {
        cursor_id: usize,
        action: CursorAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum CursorAction {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
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

    pub fn add(&mut self, file_path: Option<PathBuf>) -> BufferId {
        let buffer_id = BufferId(self.next_buffer_id);
        self.next_buffer_id += 1;
        self.buffers.push(Buffer::new(buffer_id, file_path));
        buffer_id
    }

    pub fn find_by_file_path(&self, file_path: &PathBuf) -> Option<BufferId> {
        self.buffers
            .iter()
            .find(|b| {
                if let Some(buf_path) = b.file_path.as_ref() {
                    return buf_path == file_path;
                }
                false
            })
            .map(|b| b.id)
    }

    pub fn get(&self, buffer_id: BufferId) -> Option<&Buffer> {
        self.buffers.iter().find(|buf| buf.id == buffer_id)
    }

    pub fn get_mut(&mut self, buffer_id: BufferId) -> Option<&mut Buffer> {
        self.buffers.iter_mut().find(|buf| buf.id == buffer_id)
    }

    pub fn handle_actions(&mut self, action: BuffersAction) {
        match self.get_mut(action.buffer_id) {
            Some(buffer) => {
                buffer.handle_action(action.inner_action);
            }
            None => todo!(),
        }
    }
}

pub struct Buffer {
    pub id: BufferId,
    pub content: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub cursors: Vec<Position>,
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
                    cursors: vec![Position::default()],
                }
            }
            None => Self {
                id,
                file_path: None,
                content: vec![],
                cursors: vec![Position::default()],
            },
        }
    }

    pub fn get_line(&self, index: usize) -> Option<&String> {
        self.content.get(index)
    }

    pub fn get_line_mut(&mut self, index: usize) -> Option<&mut String> {
        self.content.get_mut(index)
    }

    pub fn get_cursor(&self, cursor_id: CursorId) -> &Position {
        &self.cursors[cursor_id.0]
    }

    pub fn handle_action(&mut self, action: BufferAction) {
        match action {
            BufferAction::Save => {}
            BufferAction::CursorAction { cursor_id, action } => {
                // handle the cursor
            }
            _ => {}
        }
    }
}
