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
            Rect::new(0, 0, terminal_size.width / 2, terminal_size.height),
            Arc::clone(&buffer),
        );
        window.set_focus(true);
        self.windows.push(window);

        let window_2 = Window::new(
            1,
            Rect::new(
                terminal_size.width / 2,
                0,
                terminal_size.width,
                terminal_size.height,
            ),
            Arc::clone(&buffer),
        );
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

        queue!(self.terminal.writer, cursor::MoveTo(0, 0))?;

        for window in self.windows.iter() {
            let buffer = window.draw();

            let window_size = &window.size;

            for y in window_size.y..window_size.height - 1 {
                queue!(
                    self.terminal.writer,
                    cursor::MoveTo(window_size.x as u16, y as u16)
                )?;

                for x in window_size.x..window_size.width - 1 {
                    queue!(self.terminal.writer, style::Print(format!("{}", window.id)))?;
                }
            }
        }

        self.terminal.show_cursor()?;
        let window_cursor = self.windows.first().unwrap().get_cursor();
        self.terminal.set_cursor(window_cursor.0, window_cursor.1)?;

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
            size,
            buffer,
            cursor: (0, 0),
            focused: false,
        }
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn set_cursor(&mut self, x: u16, y: u16) {
        let x = cmp::min(x, self.size.width as u16 - 1);
        let y = cmp::min(y, self.size.height as u16 - 1);

        self.cursor = (x, y);
    }

    fn draw(&self) -> Vec<&str> {
        let mut buf: Vec<&str> = vec![];

        let Rect {
            x: start_x,
            y: start_y,
            width,
            height,
        } = self.size;

        for x in start_x..width - 1 {
            for y in start_y..height - 1 {
                if x == start_x || x == width - 1 {
                    buf.push(border::HORIZONTAL);
                } else if y == start_y || y == height - 1 {
                    buf.push(border::VERTICAL);
                } else if x == start_x && y == start_y {
                    buf.push(border::TOP_LEFT);
                } else if x == width - 1 && y == start_y {
                    buf.push(border::TOP_RIGHT);
                } else if x == start_x && y == height - 1 {
                    buf.push(border::BOTTOM_LEFT);
                } else if x == width - 1 && y == height - 1 {
                    buf.push(border::BOTTOM_RIGHT);
                } else {
                    buf.push(" ");
                }
            }
        }

        buf
    }

    fn get_cursor(&self) -> (u16, u16) {
        self.cursor
    }
}

pub struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}
impl Rect {
    fn new(x: usize, y: usize, width: usize, height: usize) -> Rect {
        Self {
            x,
            y,
            width,
            height,
        }
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
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn size(&self) -> io::Result<Rect> {
        let (width, height) = terminal::size()?;
        Ok(Rect::new(0, 0, width as usize, height as usize))
    }

    pub fn show_cursor(&mut self) -> io::Result<()> {
        execute!(self.writer, cursor::Show)?;
        Ok(())
    }

    pub fn hide_cursor(&mut self) -> io::Result<()> {
        execute!(self.writer, cursor::Hide)?;
        Ok(())
    }

    pub fn move_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        execute!(self.writer, cursor::MoveTo(x, y))?;
        Ok(())
    }

    pub fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        let (x, y) = cursor::position()?;
        Ok((x, y))
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        execute!(self.writer, cursor::MoveTo(x, y))?;
        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        execute!(self.writer, terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
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
