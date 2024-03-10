use std::{
    cmp, io,
    path::{Path, PathBuf},
};

use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    queue, terminal,
};

use crate::{
    keyboard::Keyboard,
    screen::{Direction, Position, Screen},
};

pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_path: Option<PathBuf>,
}

impl Buffer {
    pub fn new() -> Self {
        let mut args = std::env::args();
        match args.nth(1) {
            Some(file) => Buffer::from(file.as_ref()),
            None => Self {
                lines: Vec::new(),
                file_path: None,
            },
        }
    }

    fn get_line(&self, y: usize) -> Option<&Line> {
        self.lines.get(y)
    }
}

impl From<&Path> for Buffer {
    fn from(file: &Path) -> Self {
        let file_contents = std::fs::read_to_string(file).expect("Unable to read file");
        Self {
            file_path: Some(file.to_path_buf()),
            lines: file_contents.lines().map(|s| Line::new(s.into())).collect(),
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

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    buffer: Buffer,
    cursor: Position,
    offset: Position,
    // config: Rc<Config>,
}

impl Editor {
    pub fn new() -> Self {
        // let config = Rc::new(Config::new());

        Self {
            screen: Screen::new(),
            keyboard: Keyboard,
            buffer: Buffer::new(),
            cursor: Position::default(),
            offset: Position::default(),
            // rows: Rows::new(),
            // config,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        loop {
            if self.refresh_screen().is_err() {
                self.die("Error refreshing screen");
            }

            if self.process_keypress()? {
                break;
            }
        }

        self.screen.clear()?;
        queue!(self.screen.out, cursor::Show)?;
        terminal::disable_raw_mode()
    }

    pub fn refresh_screen(&mut self) -> io::Result<()> {
        self.scroll();
        self.screen.clear()?;
        self.screen.draw_rows(&self.buffer.lines, &self.offset)?;

        self.screen.draw_cursor(&self.cursor, &self.offset)?;

        self.screen.flush()
    }

    fn die<S: Into<String>>(&mut self, msg: S) {
        self.screen.clear().unwrap();
        terminal::disable_raw_mode().unwrap();
        eprintln!("{}", msg.into());
        std::process::exit(1);
    }

    /// Process the keypress and return whether the editor should exit
    fn process_keypress(&mut self) -> io::Result<bool> {
        match self.keyboard.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                return Ok(true);
            }
            KeyEvent {
                code:
                    key_code @ KeyCode::Up
                    | key_code @ KeyCode::Down
                    | key_code @ KeyCode::Left
                    | key_code @ KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            } => {
                let dir = match key_code {
                    KeyCode::Up => Direction::Up,
                    KeyCode::Down => Direction::Down,
                    KeyCode::Left => Direction::Left,
                    KeyCode::Right => Direction::Right,
                    _ => unreachable!(),
                };
                self.move_cursor(dir);
            }
            _ => {}
        };

        Ok(false)
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
            }
            Direction::Down => {
                if self.cursor.y < self.buffer.lines.len() as u16 - 1 {
                    self.cursor.y += 1;
                }
            }
            Direction::Left => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            }
            Direction::Right => {
                let cur_line = self.buffer.get_line(self.cursor.y as usize);
                if let Some(line) = cur_line {
                    if (self.cursor.x as usize) < line.render.len().saturating_sub(1) {
                        self.cursor.x += 1;
                    }
                }
            }
        }

        let cur_line = self.buffer.get_line(self.cursor.y as usize);
        if let Some(line) = cur_line {
            if (self.cursor.x as usize) > line.render.len().saturating_sub(1) {
                self.cursor.x = line.render.len().saturating_sub(1) as u16;
            }
        }
    }

    pub fn scroll(&mut self) {
        self.offset.y = cmp::min(self.offset.y, self.cursor.y);
        if self.cursor.y >= self.offset.y + self.screen.height {
            self.offset.y = self.cursor.y - self.screen.height + 1;
        }

        self.offset.x = cmp::min(self.offset.x, self.cursor.x);
        if self.cursor.x >= self.offset.x + self.screen.width {
            self.offset.x = self.cursor.x - self.screen.width + 1;
        }
    }
}
