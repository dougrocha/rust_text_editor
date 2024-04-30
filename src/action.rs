use std::path::PathBuf;

use crate::buffer::BuffersAction;

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
