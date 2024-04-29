use std::{fmt, path::PathBuf, string::ToString};

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use serde_json::Value;
use strum::Display;

use crate::buffer::BuffersAction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
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
