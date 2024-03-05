use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute, queue, style,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

const TAB_STOP: usize = 8;

fn main() -> io::Result<()> {
    let mut stdout = std::io::BufWriter::new(std::io::stderr());

    queue!(stdout, EnterAlternateScreen)?;

    let supports_keyboard_enhancement = matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    );

    if supports_keyboard_enhancement {
        queue!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )
        .expect("Failed to push keyboard enhancement flags");
    }

    let mut app = App::new();

    while app.run(&mut stdout)? {}

    stdout.flush()?;

    queue!(stdout, cursor::Show, LeaveAlternateScreen)?;

    Ok(())
}

struct App {
    reader: Reader,
    screen: Screen,
}

impl Drop for App {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Failed to disable raw mode");
        self.screen
            .clear_screen()
            .expect("Failed to clear screen when shutting down.");
    }
}

impl App {
    fn new() -> Self {
        terminal::enable_raw_mode().unwrap();

        Self {
            reader: Reader,
            screen: Screen::new(),
        }
    }

    fn process_keypress(&mut self) -> io::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            } => return Ok(false),
            KeyEvent {
                code:
                    direction @ (KeyCode::Char('j')
                    | KeyCode::Char('k')
                    | KeyCode::Char('h')
                    | KeyCode::Char('l')),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            } => self.screen.move_cursor(direction),
            _ => {}
        }

        Ok(true)
    }

    pub fn run(&mut self, w: &mut io::BufWriter<io::Stderr>) -> io::Result<bool> {
        self.screen.refresh_screen(w)?;
        self.process_keypress()
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> io::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(key_event) = event::read()? {
                    return Ok(key_event);
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
struct ScreenSize {
    rows: usize,
    cols: usize,
}

struct CursorController {
    cx: usize,
    cy: usize,

    screen_size: ScreenSize,
    row_offset: usize,
    col_offset: usize,

    render_x: usize,
}

impl CursorController {
    fn new(screen_size: ScreenSize) -> Self {
        Self {
            cx: 0,
            cy: 0,
            screen_size,
            row_offset: 0,
            col_offset: 0,
            render_x: 0,
        }
    }

    fn scroll(&mut self, rows: &EditorRows) {
        self.render_x = 0;
        if self.cy < rows.number_of_rows() {
            self.render_x = self.get_render_x(rows.get_render_row(self.cy));
        }
        // get row offset
        self.row_offset = std::cmp::min(self.row_offset, self.cy);
        if self.cy >= self.row_offset + self.screen_size.rows {
            self.row_offset = self.cy - self.screen_size.rows + 1;
        }

        // get col offset
        self.col_offset = std::cmp::min(self.col_offset, self.render_x);
        if self.render_x >= self.col_offset + self.screen_size.cols {
            self.col_offset = self.render_x - self.screen_size.cols + 1;
        }
    }

    fn move_cursor(&mut self, direction: KeyCode, rows: &EditorRows) {
        let number_of_rows = rows.number_of_rows();
        match direction {
            KeyCode::Char('j') => {
                if self.cy < number_of_rows - 1 {
                    self.cy += 1;
                }
            }
            KeyCode::Char('k') => {
                self.cy = self.cy.saturating_sub(1);
            }
            KeyCode::Char('h') => {
                if self.cx != 0 {
                    self.cx -= 1
                } else if self.cy > 0 {
                    self.cy -= 1;
                    self.cx = rows.get_row(self.cy).len();
                }
            }
            KeyCode::Char('l') => {
                if self.cy < number_of_rows && self.cx < rows.get_row(self.cy).len() {
                    self.cx += 1;
                }
            }
            _ => {}
        }

        let row_len = if self.cy < number_of_rows {
            rows.get_row(self.cy).len()
        } else {
            0
        };
        self.cx = std::cmp::min(self.cx, row_len);
    }

    fn get_render_x(&self, row: &Row) -> usize {
        row.row_content[..self.cx].chars().fold(0, |render_x, c| {
            if c == '\t' {
                render_x + (TAB_STOP - 1) - (render_x % TAB_STOP) + 1
            } else {
                render_x + 1
            }
        })
    }
}

struct Screen {
    size: ScreenSize,
    cursor: CursorController,
    rows: EditorRows,
    status_message: StatusMessage,
}

impl Screen {
    fn new() -> Self {
        let (screen_cols, screen_rows) = crossterm::terminal::size().unwrap();

        let screen_size = ScreenSize {
            rows: screen_rows as usize - 2,
            cols: screen_cols as usize,
        };

        Self {
            size: screen_size,
            cursor: CursorController::new(screen_size),
            rows: EditorRows::new(),
            status_message: StatusMessage::new("HELP: q = Quit".into()),
        }
    }

    fn clear_screen(&self) -> io::Result<()> {
        execute!(
            io::stdout(),
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )
    }

    fn move_cursor(&mut self, direction: KeyCode) {
        self.cursor.move_cursor(direction, &self.rows);
    }

    fn draw_status_bar(&self, w: &mut io::BufWriter<io::Stderr>) -> io::Result<()> {
        let mut temp = String::new();
        temp.push_str(&style::Attribute::Reverse.to_string());
        let info = format!(
            "{} --- {} lines",
            self.rows
                .filename
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("[No Name]"),
            self.rows.number_of_rows()
        );

        let info_len = std::cmp::min(info.len(), self.size.cols);
        temp.push_str(&info[..info_len]);

        let line_info = format!("{}/{}", self.cursor.cy + 1, self.rows.number_of_rows());
        for i in info_len..self.size.cols {
            if self.size.cols - i == line_info.len() {
                temp.push_str(&line_info);
                break;
            }

            temp.push(' ')
        }

        temp.push_str(&style::Attribute::Reset.to_string());
        temp.push_str("\r\n");
        queue!(w, style::Print(temp))?;

        Ok(())
    }

    fn draw_message_bar(&mut self, w: &mut io::BufWriter<io::Stderr>) -> io::Result<()> {
        queue!(w, terminal::Clear(ClearType::UntilNewLine))?;
        let mut temp = String::new();

        if let Some(msg) = self.status_message.message() {
            temp.push_str(&style::Attribute::Reverse.to_string());
            let m = &msg[..std::cmp::min(self.size.cols, msg.len())];
            temp.push_str(m);
            temp.push_str(&style::Attribute::Reset.to_string());
            queue!(w, style::Print(m))?;
        }

        Ok(())
    }

    fn refresh_screen(&mut self, w: &mut io::BufWriter<io::Stderr>) -> io::Result<()> {
        self.cursor.scroll(&self.rows);
        self.clear_screen()?;

        self.draw_rows(w)?;
        self.draw_status_bar(w)?;
        self.draw_message_bar(w)?;

        queue!(
            w,
            cursor::MoveTo(
                (self.cursor.render_x - self.cursor.col_offset) as u16,
                (self.cursor.cy - self.cursor.row_offset) as u16
            ),
            cursor::Show,
        )?;

        w.flush()
    }

    fn draw_rows(&self, w: &mut io::BufWriter<io::Stderr>) -> io::Result<()> {
        let mut temp = String::new();
        for i in 0..self.size.rows {
            let file_row = i + self.cursor.row_offset;

            if file_row >= self.rows.number_of_rows() {
                if self.rows.number_of_rows() == 0 && i == self.size.rows / 3 {
                    let welcome = format!(
                        "Welcome to my editor. Version: {}",
                        env!("CARGO_PKG_VERSION")
                    );
                    let len = self.size.cols.min(welcome.len());

                    let mut padding = (self.size.cols - len) / 2;
                    if padding != 0 {
                        temp.push_str("~");
                        padding -= 1;
                    }
                    (0..padding).for_each(|_| temp.push_str(" "));

                    temp.push_str(&welcome[..len]);
                } else {
                    temp.push_str("~");
                }
            } else {
                let row = self.rows.get_render(file_row);
                let col_offset = self.cursor.col_offset;
                let len = std::cmp::min(row.len().saturating_sub(col_offset), self.size.cols);
                let start = if len == 0 { 0 } else { col_offset };
                temp.push_str(&row[start..start + len]);
            }

            queue!(w, terminal::Clear(ClearType::UntilNewLine)).unwrap();

            temp.push_str("\r\n");
        }
        queue!(w, style::Print(temp))?;

        Ok(())
    }
}

struct EditorRows {
    row_contents: Vec<Row>,
    filename: Option<PathBuf>,
}

impl EditorRows {
    fn new() -> Self {
        let mut arg = env::args();

        match arg.nth(1) {
            None => Self {
                row_contents: Vec::new(),
                filename: None,
            },
            Some(file) => Self::from_file(file.as_ref()),
        }
    }

    fn from_file(file: &Path) -> Self {
        let file_contents = std::fs::read_to_string(file).expect("Unable to read file");
        Self {
            filename: Some(file.to_path_buf()),
            row_contents: file_contents
                .lines()
                .map(|it| {
                    let mut row = Row::new(it.into(), String::new());
                    Self::render_row(&mut row);
                    row
                })
                .collect(),
        }
    }

    fn number_of_rows(&self) -> usize {
        self.row_contents.len()
    }

    fn get_row(&self, at: usize) -> &str {
        &self.row_contents[at].row_content
    }

    fn get_render(&self, at: usize) -> &String {
        &self.row_contents[at].render
    }

    fn get_render_row(&self, at: usize) -> &Row {
        &self.row_contents[at]
    }

    fn render_row(row: &mut Row) {
        let mut index = 0;
        let capacity = row
            .row_content
            .chars()
            .fold(0, |acc, next| acc + if next == '\t' { TAB_STOP } else { 1 });
        row.render = String::with_capacity(capacity);
        row.row_content.chars().for_each(|c| {
            index += 1;
            if c == '\t' {
                row.render.push_str(" ");
                while index % TAB_STOP != 0 {
                    row.render.push_str(" ");
                    index += 1;
                }
            } else {
                row.render.push(c);
            }
        });
    }
}

struct Row {
    row_content: Box<str>,
    render: String,
}

impl Row {
    fn new(row_content: Box<str>, render: String) -> Self {
        Self {
            row_content,
            render,
        }
    }
}

struct StatusMessage {
    message: Option<String>,
    set_time: Option<Instant>,
}

impl StatusMessage {
    fn new(initial_message: String) -> Self {
        Self {
            message: Some(initial_message),
            set_time: Some(Instant::now()),
        }
    }

    fn set_message(&mut self, message: String) {
        self.message = Some(message);
        self.set_time = Some(Instant::now())
    }

    fn message(&mut self) -> Option<&String> {
        self.set_time.and_then(|time| {
            if time.elapsed() > Duration::from_secs(5) {
                self.message = None;
                self.set_time = None;
                None
            } else {
                Some(self.message.as_ref().unwrap())
            }
        })
    }
}
