use std::{fs, path::PathBuf};

use color_eyre::eyre::Result;
use ratatui::layout::Rect;

use crate::{components::Component, tui};

#[derive(Default)]
pub struct Buffer {
    pub id: usize,
    pub content: Vec<String>,
    pub file_path: Option<PathBuf>,
}

impl Buffer {
    pub fn new(id: usize, file_path: Option<&PathBuf>) -> Self {
        match file_path {
            Some(file_path) => {
                let content = fs::read_to_string(file_path).unwrap();
                let content = content.lines().map(|line| line.to_string()).collect();

                Self {
                    id,
                    file_path: Some(file_path.to_path_buf()),
                    content,
                }
            }
            None => Self::default(),
        }
    }

    pub fn get_line(&self, index: usize) -> Option<&String> {
        self.content.get(index)
    }
}

impl Component for Buffer {
    fn draw(&mut self, f: &mut tui::Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
}
