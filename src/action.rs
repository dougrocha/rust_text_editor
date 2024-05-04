use std::path::PathBuf;

use crate::{buffer::BufferId, window::CursorId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Help,

    // Window Action
    OpenFile(PathBuf),

    // Buffer Actions
    Buffer(BuffersAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuffersAction {
    pub buffer_id: BufferId,
    pub inner_action: BufferAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferAction {
    Save,
    CursorAction {
        cursor_id: CursorId,
        action: CursorAction,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorAction {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    InsertChar(char),
    EndOfLine,
}

impl From<BuffersAction> for Action {
    fn from(action: BuffersAction) -> Action {
        Action::Buffer(action)
    }
}
