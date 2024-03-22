use std::{
    cmp, io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor,
    event::KeyCode,
    execute, queue, style,
    terminal::{self, ClearType},
};

mod event;

use event::{Event, EventHandler};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = Editor::new()?;

    app.run().await?;

    Ok(())
}

enum Mode {
    Normal,
    Insert,
    Visual,
}

struct Editor {
    terminal: Terminal<io::Stdout>,
    event_handler: EventHandler,

    buffers: Vec<Arc<Mutex<Buffer>>>,

    windows: Vec<Window>,

    mode: Mode,
    cursor: (u16, u16),

    is_running: bool,
}

struct Line {
    content: Box<str>,
    render: String,
}

impl Line {
    pub fn new(content: Box<str>) -> Self {
        Self {
            render: content.to_string(),
            content,
        }
    }
}

struct Buffer {
    file_path: Option<PathBuf>,

    content: Vec<Line>,
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
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        let event_handler = EventHandler::new(100);

        Ok(Self {
            terminal: Terminal::new(io::stdout()),
            event_handler,

            buffers: vec![],
            windows: vec![],

            mode: Mode::Normal,

            cursor: (0, 0),

            is_running: true,
        })
    }

    pub fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        self.terminal.show_cursor()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.init()?;

        let buffer = Arc::new(Mutex::new(Buffer::from_file(PathBuf::from("test.txt"))?));
        self.buffers.push(Arc::clone(&buffer));

        let terminal_size = self.terminal.size()?;

        let mut window = Window::new(
            0,
            Rect::new(
                terminal_size.start,
                Vec2::new(terminal_size.width() / 2, terminal_size.height()),
            ),
            Arc::clone(&buffer),
        );
        window.set_focus(false);
        self.windows.push(window);

        let mut window_2 = Window::new(
            1,
            Rect::new(Vec2::new(terminal_size.width() / 2, 0), terminal_size.end),
            Arc::clone(&buffer),
        );

        window_2.set_focus(true);
        self.windows.push(window_2);

        loop {
            self.draw()?;

            match self.event_handler.next().await? {
                Event::Tick => {}
                Event::Key(key_event) => {
                    let window = self.windows.iter_mut().find(|x| x.focused).unwrap();

                    let (x, y) = window.cursor;

                    if key_event.code == KeyCode::Char('q') {
                        self.is_running = false;
                    } else if key_event.code == KeyCode::Down {
                        window.set_cursor(x, y + 1);
                    } else if key_event.code == KeyCode::Up {
                        window.set_cursor(x, y.saturating_sub(1));
                    } else if key_event.code == KeyCode::Left {
                        window.set_cursor(x.saturating_sub(1), y);
                    } else if key_event.code == KeyCode::Right {
                        window.set_cursor(x + 1, y);
                    }
                }
                Event::Mouse(_) => {}
                Event::Resize(_x, _y) => {
                    // self.terminal.resize((x, y));
                }
            }

            if !self.is_running {
                break;
            }
        }

        self.cleanup()?;

        Ok(())
    }

    pub fn draw(&mut self) -> io::Result<()> {
        self.terminal.clear()?;

        self.terminal.hide_cursor()?;

        for window in self.windows.iter() {
            let Rect {
                start: Vec2 { x: left, y: top },
                end:
                    Vec2 {
                        x: width,
                        y: height,
                    },
            } = window.size;

            for y in top..height {
                queue!(self.terminal.writer, cursor::MoveTo(left as u16, y as u16))?;
                for _x in left..width {
                    queue!(self.terminal.writer, style::Print(format!("{}", window.id)))?;
                }
            }
        }

        self.terminal.show_cursor()?;
        let (x, y) = self.windows.iter().find(|x| x.focused).unwrap().cursor;
        self.terminal.set_cursor(x, y)?;

        self.terminal.flush()?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

struct Window {
    id: i32,

    size: Rect,
    buffer: Arc<Mutex<Buffer>>,

    cursor: (u16, u16),

    focused: bool,
}

impl Window {
    fn new(id: i32, size: Rect, buffer: Arc<Mutex<Buffer>>) -> Self {
        Self {
            id,
            cursor: (size.start.x as u16, size.start.y as u16),

            size,
            buffer,
            focused: false,
        }
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn set_cursor(&mut self, x: u16, y: u16) {
        if x >= self.size.end.x as u16 {
            self.cursor.0 = self.size.end.x as u16 - 1;
        } else {
            self.cursor.0 = cmp::max(x, self.size.start.x as u16);
        }

        if y >= self.size.end.y as u16 {
            self.cursor.1 = self.size.end.y as u16 - 1;
        } else {
            self.cursor.1 = cmp::max(y, self.size.start.y as u16);
        }
    }

    fn get_cursor(&self) -> (u16, u16) {
        self.cursor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vec2 {
    x: usize,
    y: usize,
}

impl Vec2 {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    fn width(&self) -> usize {
        self.x
    }

    fn height(&self) -> usize {
        self.y
    }
}

#[derive(Debug)]
pub struct Rect {
    /// The start position of the rectangle
    ///
    /// Normally the top-left corner
    start: Vec2,

    /// The end position of the rectangle
    ///
    /// Normally the bottom-right corner
    end: Vec2,
}

impl Rect {
    fn new(start: Vec2, end: Vec2) -> Rect {
        Self { start, end }
    }

    fn width(&self) -> usize {
        self.end.x - self.start.x
    }

    fn height(&self) -> usize {
        self.end.y - self.start.y
    }
}

pub struct Terminal<W>
where
    W: io::Write,
{
    writer: W,
}

impl<W> Terminal<W>
where
    W: io::Write,
{
    fn new(writer: W) -> Self {
        Self { writer }
    }

    fn size(&self) -> io::Result<Rect> {
        let (width, height) = terminal::size()?;
        Ok(Rect {
            start: Vec2::zero(),
            end: Vec2::new(width as usize, height as usize),
        })
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        execute!(self.writer, cursor::Show)?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        execute!(self.writer, cursor::Hide)?;
        Ok(())
    }

    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        let (x, y) = cursor::position()?;
        Ok((x, y))
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        execute!(self.writer, cursor::MoveTo(x, y))?;
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        execute!(self.writer, terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

mod border {
    pub const VERTICAL: &str = "│";
    pub const HORIZONTAL: &str = "─";

    pub const TOP_LEFT: &str = "┌";
    pub const TOP_RIGHT: &str = "┐";
    pub const BOTTOM_LEFT: &str = "└";
    pub const BOTTOM_RIGHT: &str = "┘";
}
