use std::{
    cmp,
    fs::File,
    io::{self, Read, Write},
    time,
};

use crossterm::{
    cursor,
    event::{self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue, style,
    terminal::{self, EnterAlternateScreen},
};

fn main() -> io::Result<()> {
    let mut app = Editor::new();

    app.setup()?;

    while !app.run()? {}

    Ok(())
}

struct Editor {
    key_reader: KeyReader,
    cursor_controller: CursorController,

    lines: Vec<String>,
}

impl Drop for Editor {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen).unwrap();
    }
}

impl Editor {
    fn new() -> Self {
        let terminal_size = terminal::size().unwrap();

        Self {
            key_reader: KeyReader,
            cursor_controller: CursorController::new(terminal_size),

            lines: Vec::new(),
        }
    }

    fn setup(&self) -> io::Result<()> {
        terminal::enable_raw_mode().unwrap();

        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            terminal::SetSize(
                self.cursor_controller.screen.0 as u16,
                self.cursor_controller.screen.1 as u16,
            ),
            cursor::MoveTo(0, 0)
        )?;

        Ok(())
    }

    fn clear_screen(&self) -> io::Result<()> {
        queue!(
            io::stdout(),
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
    }

    fn run(&mut self) -> io::Result<bool> {
        let mut stdout = io::stdout();

        self.read_file().unwrap();

        loop {
            if !self.handle_key()? {
                break;
            }

            self.cursor_controller.scroll();
            self.clear_screen()?;

            self.draw_rows()?;

            queue!(
                stdout,
                cursor::Show,
                cursor::MoveTo(
                    self.cursor_controller.pos.0 as u16
                        - self.cursor_controller.pos_offset.0 as u16,
                    self.cursor_controller.pos.1 as u16
                        - self.cursor_controller.pos_offset.1 as u16
                )
            )?;

            stdout.flush()?;
        }

        Ok(true)
    }

    fn handle_key(&mut self) -> io::Result<bool> {
        match self.key_reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                return Ok(false);
            }
            KeyEvent {
                code:
                    dir @ KeyCode::Up | dir @ KeyCode::Down | dir @ KeyCode::Left | dir @ KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            } => {
                let direction = match dir {
                    KeyCode::Up => Direction::Up,
                    KeyCode::Down => Direction::Down,
                    KeyCode::Left => Direction::Left,
                    KeyCode::Right => Direction::Right,
                    _ => unreachable!(),
                };
                self.cursor_controller.move_cursor(direction, &self.lines);
            }
            _ => {}
        };

        Ok(true)
    }

    fn read_file(&mut self) -> io::Result<()> {
        let mut file = File::open("src/main.rs")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        self.lines = contents.lines().map(|s| s.to_string()).collect();
        Ok(())
    }

    fn draw_rows(&self) -> io::Result<()> {
        let lines = &self.lines;

        for i in 0..self.cursor_controller.screen.1 {
            if i + self.cursor_controller.pos_offset.1 >= lines.len() {
                break;
            }

            let line = &lines[i + self.cursor_controller.pos_offset.1];
            let col_offet = self.cursor_controller.pos_offset.0;

            let line_len = cmp::min(
                line.len()
                    .saturating_sub(self.cursor_controller.pos_offset.0),
                self.cursor_controller.screen.0,
            );
            let start = if line_len == 0 { 0 } else { col_offet };

            queue!(
                io::stdout(),
                cursor::MoveTo(0, i as u16),
                terminal::Clear(terminal::ClearType::CurrentLine),
                style::Print(&line[start..start + line_len]),
            )?;
        }

        Ok(())
    }
}

struct KeyReader;

impl KeyReader {
    fn read_key(&self) -> io::Result<KeyEvent> {
        loop {
            if event::poll(time::Duration::from_millis(500))? {
                if let Event::Key(key_event) = event::read()? {
                    return Ok(key_event);
                }
            }
        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct CursorController {
    screen: (usize, usize),

    pos: (usize, usize),

    pos_offset: (usize, usize),
}

impl CursorController {
    fn new(screen: (u16, u16)) -> Self {
        Self {
            pos: (0, 0),
            pos_offset: (0, 0),

            screen: (screen.0 as usize, screen.1 as usize),
        }
    }

    fn move_cursor(&mut self, direction: Direction, lines: &[String]) {
        match direction {
            Direction::Up => {
                self.pos.1 = self.pos.1.saturating_sub(1);
            }
            Direction::Down => {
                if self.pos.1 < lines.len().saturating_sub(1) {
                    self.pos.1 += 1;
                }
            }
            Direction::Left => {
                self.pos.0 = self.pos.0.saturating_sub(1);
            }
            Direction::Right => {
                if self.pos.0 < lines[self.pos.1].len().saturating_sub(1) {
                    self.pos.0 += 1;
                }
            }
        }

        if self.pos.0 > lines[self.pos.1].len() {
            self.pos.0 = lines[self.pos.1].len().saturating_sub(1);
        }
    }

    fn scroll(&mut self) {
        self.pos_offset.1 = cmp::min(self.pos_offset.1, self.pos.1);
        if self.pos.1 >= self.pos_offset.1 + self.screen.1 {
            self.pos_offset.1 = self.pos.1 - self.screen.1 + 1;
        }

        self.pos_offset.0 = cmp::min(self.pos_offset.0, self.pos.0);
        if self.pos.0 >= self.pos_offset.0 + self.screen.0 {
            self.pos_offset.0 = self.pos.0 - self.screen.0 + 1;
        }
    }
}
