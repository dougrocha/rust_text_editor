use color_eyre::Result;
use std::path::PathBuf;

use crate::Vec2;

pub struct Line {
    content: Box<str>,
    pub render: String,
}

impl Line {
    pub fn new(content: Box<str>) -> Self {
        Self {
            render: content.to_string(),
            content,
        }
    }
}

#[derive(Default)]
pub struct Buffer {
    pub file_path: Option<PathBuf>,

    pub content: Vec<Line>,

    pub is_dirty: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: vec![],
            is_dirty: false,
        }
    }

    pub fn from_file(file_path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&file_path)?;
        let content = content.lines().map(|line| Line::new(line.into())).collect();

        Ok(Self {
            file_path: Some(file_path),
            content,
            is_dirty: false,
        })
    }

    pub fn get_line(&self, index: usize) -> Option<&Line> {
        self.content.get(index)
    }

    pub fn get_line_mut(&mut self, index: usize) -> Option<&mut Line> {
        self.content.get_mut(index)
    }

    pub fn insert_character(&mut self, c: char, x: u16, y: u16) -> (u16, u16) {
        let x = x as usize;
        let y = y as usize;

        if y == self.content.len() {
            self.content.push(Line::new("".into()));
        }

        let line = self.get_line_mut(y).unwrap();
        let at = std::cmp::min(x, line.render.len());

        line.render.insert(at, c);
        self.is_dirty = true;

        (x as u16 + 1, y as u16)
    }

    pub fn delete_character(&mut self, x: u16, y: u16) -> (u16, u16) {
        let x = x as usize;
        let y = y as usize;

        if x == 0 && y == 0 {
            return (0, 0);
        }

        if x == 0 && y != 0 {
            let line = self.content.remove(y).render;
            let prev_line = self.get_line_mut(y - 1).unwrap();

            let prev_line_len = prev_line.render.len();

            prev_line.render.push_str(&line);

            return (prev_line_len as u16, y as u16 - 1);
        }

        self.get_line_mut(y).unwrap().render.remove(x - 1);

        self.is_dirty = true;

        (x as u16 - 1, y as u16)
    }

    pub fn insert_newline(&mut self, x: u16, y: u16) -> (u16, u16) {
        let x = x as usize;
        let y = y as usize;

        let line = self.get_line_mut(y).unwrap();
        let line_len = line.render.len();

        if x == line_len {
            self.content.insert(y + 1, Line::new("".into()));
            return (0, y as u16 + 1);
        }

        if x == 0 {
            self.content.insert(y, Line::new("".into()));
            return (0, y as u16 + 1);
        }

        let right = line.render.split_off(x);
        self.content.insert(y + 1, Line::new(right.into()));

        (0, y as u16 + 1)
    }
}
