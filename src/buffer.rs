use std::io;
use std::path::{Path, PathBuf};

enum BufferState {
    Active,
    Hidden,
}

pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_path: Option<PathBuf>,

    state: BufferState,

    // if the buffer has been modified
    pub dirty: bool,
}

impl Buffer {
    pub fn new() -> Self {
        let mut args = std::env::args();
        match args.nth(1) {
            Some(file) => Buffer::from(file.as_ref()),
            None => Self {
                lines: Vec::new(),
                file_path: None,
                state: BufferState::Active,
                dirty: false,
            },
        }
    }

    pub fn save(&mut self) -> io::Result<(usize, Option<&str>)> {
        if let Some(file_path) = &self.file_path {
            if !self.dirty {
                return Ok((0, file_path.to_str()));
            }

            let contents = self
                .lines
                .iter()
                .map(|l| l.render.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            let bytes_written = contents.as_bytes().len();

            // TODO: Handle file that doesnt exist before hand
            std::fs::write(file_path, contents)?;

            self.dirty = false;
            Ok((bytes_written, file_path.to_str()))
        } else {
            // TODO: handle save_as case here
            Ok((0, None))
        }
    }

    pub fn get_line(&self, y: usize) -> Option<&Line> {
        self.lines.get(y)
    }

    pub fn get_line_mut(&mut self, y: usize) -> Option<&mut Line> {
        self.lines.get_mut(y)
    }
}

impl From<&Path> for Buffer {
    fn from(file: &Path) -> Self {
        let file_contents = std::fs::read_to_string(file).expect("Unable to read file");
        Self {
            file_path: Some(file.to_path_buf()),
            lines: file_contents.lines().map(|s| Line::new(s.into())).collect(),
            dirty: false,
            state: BufferState::Active,
        }
    }
}

pub struct Line {
    pub content: Box<str>,
    pub render: String,
}

const TAB_STOP: usize = 8;

impl Line {
    pub fn new(content: Box<str>) -> Self {
        let render = content.chars().fold(String::new(), |mut acc, c| {
            if c == '\t' {
                let spaces = TAB_STOP - (acc.len() % TAB_STOP);
                acc.push_str(&" ".repeat(spaces));
            } else {
                acc.push(c);
            }
            acc
        });

        Self { content, render }
    }
}
