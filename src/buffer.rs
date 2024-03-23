use std::{io, path::PathBuf};

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

pub struct Buffer {
    pub file_path: Option<PathBuf>,

    pub content: Vec<Line>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: vec![],
        }
    }

    pub fn from_file(file_path: PathBuf) -> io::Result<Self> {
        let content = std::fs::read_to_string(&file_path)?;
        let content = content.lines().map(|line| Line::new(line.into())).collect();

        Ok(Self {
            file_path: Some(file_path),
            content,
        })
    }

    pub fn get_line(&self, index: usize) -> Option<&Line> {
        self.content.get(index)
    }
}
