use std::{cmp, collections::VecDeque, io, time};

use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue, terminal,
};

use crate::{
    buffer::{Buffer, Line},
    keyboard::Keyboard,
    screen::{Direction, Position, Screen},
    status_message::Message,
};

pub type Messages = VecDeque<Message>;

pub enum Mode {
    Normal,
    Insert,
    Command,
    Visual,
    VisualLine,
}

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    buffer: Buffer,
    cursor: Position,
    offset: Position,
    messages: Messages,

    mode: Mode,
    // config: Rc<Config>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            screen: Screen::new(),
            keyboard: Keyboard,
            buffer: Buffer::new(),
            cursor: Position::default(),
            offset: Position::default(),
            messages: VecDeque::new(),
            mode: Mode::Normal,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        self.add_message(Message::new(
            "HELP: Ctrl-S = save | Ctrl-Q = quit".to_string(),
        ));

        // change cursor style depending on config or command/insert mode
        execute!(self.screen.out, cursor::SetCursorStyle::SteadyBar)?;

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
        self.screen
            .draw_gutter(&self.buffer.lines, &self.cursor, &self.offset)?;
        self.screen.draw_rows(&self.buffer.lines, &self.offset)?;
        self.screen
            .draw_status_bar(&self.buffer, &self.cursor, &self.mode)?;

        self.purge_messages();
        self.screen.draw_message(&mut self.messages)?;

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
        let key_event = self.keyboard.read_key()?;

        match self.mode {
            Mode::Normal => match key_event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    return Ok(true);
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press,
                    ..
                } => match c {
                    ':' => self.mode = Mode::Command,
                    'V' => self.mode = Mode::VisualLine,
                    _ => {}
                },
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                } => match c {
                    'j' => self.move_cursor(Direction::Down),
                    'k' => self.move_cursor(Direction::Up),
                    'h' => self.move_cursor(Direction::Left),
                    'l' => self.move_cursor(Direction::Right),

                    'i' => self.mode = Mode::Insert,
                    'v' => self.mode = Mode::Visual,
                    _ => {}
                },
                KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                } => self.mode = Mode::Normal,
                _ => {}
            },
            Mode::Insert => {
                if let KeyEvent {
                    code,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                } = key_event
                {
                    match code {
                        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                            let dir = match code {
                                KeyCode::Up => Direction::Up,
                                KeyCode::Down => Direction::Down,
                                KeyCode::Left => Direction::Left,
                                KeyCode::Right => Direction::Right,
                                _ => unreachable!(),
                            };
                            self.move_cursor(dir);
                        }
                        KeyCode::Char(c) => self.insert_character(c),
                        KeyCode::Backspace => self.delete_character(),
                        KeyCode::Enter => self.insert_newline(),
                        KeyCode::Esc => self.mode = Mode::Normal,
                        _ => {}
                    }
                }
            }
            Mode::Command => {
                if let KeyEvent {
                    code,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                } = key_event
                {
                    if KeyCode::Esc == code {
                        self.mode = Mode::Normal
                    }
                }
            }
            Mode::Visual => {
                if let KeyEvent {
                    code,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                } = key_event
                {
                    if KeyCode::Esc == code {
                        self.mode = Mode::Normal
                    }
                }
            }
            Mode::VisualLine => {
                if let KeyEvent {
                    code,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                } = key_event
                {
                    if KeyCode::Esc == code {
                        self.mode = Mode::Normal
                    }
                }
            }
        }

        if let KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            ..
        } = key_event
        {
            match self.buffer.save() {
                Ok((bytes, file)) => {
                    let message_text = format!(
                        "{:?} bytes written to {:?}",
                        bytes,
                        file.unwrap().to_string()
                    );
                    self.add_message(Message::with_duration(
                        message_text,
                        time::Duration::from_secs(2),
                    ))
                }
                Err(_) => self.add_message(Message::with_duration(
                    "Error saving file".to_string(),
                    time::Duration::from_secs(2),
                )),
            }
        };

        Ok(false)
    }

    fn insert_character(&mut self, c: char) {
        if (self.cursor.y as usize) == self.buffer.lines.len() {
            self.buffer.lines.push(Line::new("".into()));
        }

        let line = self.buffer.get_line_mut(self.cursor.y as usize).unwrap();

        let at = cmp::min(self.cursor.x as usize, line.render.len());

        line.render.insert(at, c);

        self.cursor.x += 1;

        self.buffer.dirty = true;
    }

    fn delete_character(&mut self) {
        if self.cursor.y == 0 && self.cursor.x == 0 {
            return;
        }

        if self.cursor.x == 0 {
            let cur_line = self.buffer.lines.remove(self.cursor.y as usize).render;

            let prev_line = self
                .buffer
                .get_line_mut(self.cursor.y as usize - 1)
                .unwrap();
            let prev_len = prev_line.render.len();

            prev_line.render.push_str(&cur_line);

            self.cursor.y -= 1;
            self.cursor.x = prev_len as u16;

            return;
        }

        self.buffer
            .get_line_mut(self.cursor.y as usize)
            .unwrap()
            .render
            .remove(self.cursor.x as usize - 1);
        self.cursor.x -= 1;

        self.buffer.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let cur_line = self.buffer.get_line_mut(self.cursor.y as usize).unwrap();
        let new_line = cur_line.render.split_off(self.cursor.x as usize);
        self.buffer.lines.insert(
            self.cursor.y as usize + 1,
            Line::new(new_line.into_boxed_str()),
        );
        self.cursor.y += 1;
        self.cursor.x = 0;

        self.buffer.dirty = true;
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
                    if (self.cursor.x as usize) < line.render.len() {
                        self.cursor.x += 1;
                    }
                }
            }
        }

        let cur_line = self.buffer.get_line(self.cursor.y as usize);
        if let Some(line) = cur_line {
            if (self.cursor.x as usize) > line.render.len() {
                self.cursor.x = line.render.len().saturating_sub(1) as u16;
            }
        }
    }

    pub fn scroll(&mut self) {
        let scrolloff = 8;

        self.offset.y = cmp::min(self.offset.y, self.cursor.y.saturating_sub(scrolloff));
        if self.cursor.y >= self.offset.y + self.screen.height.saturating_sub(scrolloff)
            && self.buffer.lines.len() != (self.offset.y + self.screen.height) as usize
        {
            self.offset.y = self
                .cursor
                .y
                .saturating_sub(self.screen.height.saturating_sub(scrolloff))
                + 1;
        }

        self.offset.x = cmp::min(self.offset.x, self.cursor.x);
        if self.cursor.x >= self.offset.x + self.screen.width {
            self.offset.x = self.cursor.x - self.screen.width + 1;
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
    }

    fn purge_messages(&mut self) {
        if let Some(message) = self.messages.front() {
            if message.has_expired() {
                self.messages.pop_front();
            }
        }
    }
}
