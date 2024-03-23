use std::{
    cell::RefCell,
    cmp, io,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};

use buffer::Buffer;
use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
};

mod buffer;
mod event;
mod symbols;
mod window;

use event::{Event, EventHandler};
use window::{Rect, Vec2, Window};

use crate::window::Cell;

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
    tick_rate: f64,
    frame_time: f64,

    terminal: Terminal<io::Stdout>,
    event_handler: EventHandler,

    buffers: Vec<Rc<RefCell<Buffer>>>,

    windows: Vec<Window>,

    mode: Mode,
    cursor: (u16, u16),

    is_running: bool,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        let event_handler = EventHandler::new(100);

        Ok(Self {
            tick_rate: 1.0,
            frame_time: 30.0,

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
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.init()?;

        let buffer = Rc::new(RefCell::new(Buffer::from_file(PathBuf::from("test.txt"))?));
        self.buffers.push(Rc::clone(&buffer));

        let terminal_size = self.terminal.size()?;

        let mut window = Window::new(
            0,
            Rect::new(
                terminal_size.start,
                Vec2::new(terminal_size.width() / 2, terminal_size.height()),
            ),
            Rc::clone(&buffer),
        );
        window.set_focus(false);
        self.windows.push(window);

        let mut window_2 = Window::new(
            1,
            Rect::new(Vec2::new(terminal_size.width() / 2, 0), terminal_size.end),
            Rc::clone(&buffer),
        );

        window_2.set_focus(true);
        self.windows.push(window_2);

        loop {
            self.draw()?;

            match self.event_handler.next().await? {
                Event::Tick => {}
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Char('q') {
                        self.is_running = false;
                    }

                    if let Some(window) = self.windows.iter_mut().find(|x| x.is_focused()) {
                        window.handle_key_event(key_event);
                        window.scroll();
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
        self.terminal.flush()?;

        Ok(())
    }

    pub fn draw(&mut self) -> io::Result<()> {
        self.terminal.hide_cursor()?;

        let terminal_size = self.terminal.size()?;

        let mut frame = Frame::new(terminal_size);

        for window in self.windows.iter_mut() {
            window.draw(&mut frame);
        }

        self.terminal.set_cursor(0, 0)?;
        for cell in frame.cells.iter() {
            let char = cell.symbol();
            queue!(self.terminal.writer, style::Print(char))?;
        }

        if let Some(focused_window) = self.windows.iter().find(|x| x.is_focused()) {
            let (x, y) = focused_window.get_cursor();
            let top = focused_window.size.top();
            let left = focused_window.size.left();
            let (offset_x, offset_y) = focused_window.offset;

            self.terminal.show_cursor()?;
            self.terminal.set_cursor(
                x - offset_x as u16 + left as u16,
                y - offset_y as u16 + top as u16,
            )?;
        }

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

pub struct Frame {
    pub size: Rect,
    pub cells: Vec<Cell>,
}

impl Frame {
    pub fn new(size: Rect) -> Self {
        let cells = vec![Cell::default(); size.area()];
        Self { size, cells }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, char: &str) {
        let index = y * self.size.width() + x;
        if let Some(cell) = self.cells.get_mut(index) {
            cell.set_cell(char);
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        let index = y * self.size.width() + x;
        &self.cells[index]
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
        queue!(self.writer, cursor::Show)?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        queue!(self.writer, cursor::Hide)?;
        Ok(())
    }

    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        let (x, y) = cursor::position()?;
        Ok((x, y))
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        queue!(self.writer, cursor::MoveTo(x, y))?;
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        queue!(self.writer, terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
