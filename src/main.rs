use std::{cell::RefCell, path::PathBuf, rc::Rc};

use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::{event::KeyCode, queue, style};
use editor::{
    action::Action,
    buffer::Buffer,
    component::Component,
    event::{self, Event},
    frame::Frame,
    terminal::Terminal,
    window::Window,
    Rect,
};
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(name = "rim")]
#[command(about = "A wack rust text editor", long_about = None)]
struct Cli {
    file_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut app = if let Some(file_path) = cli.file_path {
        Editor::with_file(file_path)
    } else {
        Editor::new()
    }?;

    app.run().await?;

    Ok(())
}

struct Editor {
    terminal: Terminal,

    buffers: Vec<Rc<RefCell<Buffer>>>,

    components: Vec<Box<dyn Component>>,

    is_running: bool,

    viewport: Rect,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let terminal_event_handler = event::EventHandler::new(4.0, 60.0);
        let terminal = Terminal::new(terminal_event_handler);

        Ok(Self {
            terminal,

            buffers: vec![],
            components: vec![],

            is_running: true,
            viewport: Rect::ZERO,
        })
    }

    pub fn with_file(file_path: PathBuf) -> Result<Self> {
        let terminal_event_handler = event::EventHandler::new(4.0, 60.0);
        let terminal = Terminal::new(terminal_event_handler);

        let buffer = Rc::new(RefCell::new(Buffer::from_file(file_path)?));

        let mut window = Window::new(0, terminal.size()?, Rc::clone(&buffer));
        window.set_focus(true);

        Ok(Self {
            terminal,

            buffers: vec![buffer],
            components: vec![Box::new(window)],

            is_running: true,
            viewport: Rect::ZERO,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

        self.terminal.start()?;
        self.viewport = self.terminal.size()?;

        for window in self.components.iter_mut() {
            window.register_action_handler(action_tx.clone())?;
        }

        for window in self.components.iter_mut() {
            window.init()?;
        }

        loop {
            if let Some(e) = self.terminal.event_handler.next().await {
                match e {
                    Event::Exit => action_tx.send(Action::Quit)?,
                    Event::Tick => action_tx.send(Action::Tick)?,
                    Event::Render => action_tx.send(Action::Render)?,
                    Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    Event::Key(key_event) => {
                        if key_event.code == KeyCode::Char('q') {
                            action_tx.send(Action::Quit)?;
                        }

                        action_tx.send(Action::Key(key_event))?
                    }

                    _ => {}
                }

                for window in self.components.iter_mut() {
                    if let Some(action) = window.handle_events(Some(e.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                match action {
                    Action::Quit => self.is_running = false,
                    Action::Render => self.draw()?,
                    Action::Tick => {}
                    Action::Resize(x, y) => {
                        self.viewport = Rect::new(0, 0, x as usize, y as usize);
                    }
                    Action::Key(key_event) => {
                        for component in self.components.iter_mut() {
                            component.handle_key_events(key_event)?;
                        }
                    }
                    _ => {}
                }
            }

            if !self.is_running {
                self.terminal.stop()?;
                break;
            }
        }

        self.terminal.cleanup()?;

        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let mut frame = Frame::new(self.viewport);

        for window in self.components.iter_mut() {
            window.draw(&mut frame)?;
        }

        let cursor_position = frame.cursor_position;

        self.terminal.hide_cursor()?;
        self.flush(&frame)?;

        match cursor_position {
            None => {
                self.terminal.hide_cursor()?;
            }
            Some((x, y)) => {
                self.terminal.set_cursor(x, y)?;
                self.terminal.show_cursor()?;
            }
        }

        self.terminal.flush()?;

        Ok(())
    }

    fn flush(&mut self, frame: &Frame) -> Result<()> {
        queue!(self.terminal.out, crossterm::cursor::MoveTo(0, 0))?;
        for cell in frame.cells.iter() {
            let char = cell.symbol();
            queue!(self.terminal.out, style::Print(char))?;
        }

        Ok(())
    }
}
