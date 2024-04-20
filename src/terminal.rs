use std::io::{self, Write};

use crate::{event::EventHandler, Rect, Vec2};
use crossterm::{
    cursor, execute,
    terminal::{self, ClearType},
};

pub struct Terminal {
    pub out: io::Stdout,

    pub event_handler: EventHandler,
}

impl Terminal {
    pub fn new(event_handler: EventHandler) -> Self {
        let out = io::stdout();
        Self { out, event_handler }
    }

    pub fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;

        self.hide_cursor()?;
        self.clear()?;

        Ok(())
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.event_handler.start();
        self.flush()?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;

        self.show_cursor()?;
        self.flush()?;

        Ok(())
    }

    pub fn stop(&mut self) -> io::Result<()> {
        self.event_handler.stop()?;
        self.flush()?;

        Ok(())
    }

    pub fn size(&self) -> io::Result<Rect> {
        let (width, height) = terminal::size()?;
        Ok(Rect {
            min: Vec2::ZERO,
            max: Vec2::new(width as usize, height as usize),
        })
    }

    pub fn show_cursor(&mut self) -> io::Result<()> {
        execute!(self.out, cursor::Show)?;
        Ok(())
    }

    pub fn hide_cursor(&mut self) -> io::Result<()> {
        execute!(self.out, cursor::Hide)?;
        Ok(())
    }

    pub fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        let (x, y) = cursor::position()?;
        Ok((x, y))
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        execute!(self.out, cursor::MoveTo(x, y))?;
        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        execute!(self.out, terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()?;
        Ok(())
    }
}
