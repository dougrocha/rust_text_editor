use crossterm::{cursor, queue, style, terminal};
use std::{
    cmp,
    io::{self, Write},
};

use crate::{
    editor::{Buffer, Line},
    status_message::Message,
};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub struct StatusBar {
    pub height: u16,
    pub width: u16,
}

pub struct Screen {
    pub out: io::Stdout,

    pub width: u16,
    pub height: u16,

    pub gutter_size: usize,

    pub status_bar: StatusBar,
}

impl Screen {
    pub fn new() -> Self {
        let (columns, rows) = terminal::size().unwrap();

        let status_bar = StatusBar {
            height: 1,
            width: columns,
        };

        Self {
            out: io::stdout(),
            width: columns,
            height: rows - status_bar.height,

            gutter_size: 1,

            status_bar,
        }
    }

    pub fn clear(&mut self) -> io::Result<()> {
        queue!(
            self.out,
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }

    pub fn draw_rows(&mut self, lines: &[Line], offset: &Position) -> io::Result<()> {
        for i in 0..self.height {
            let row = i as usize + offset.y as usize;

            queue!(self.out, cursor::MoveTo(0, i))?;
            queue!(io::stdout(), style::Print("~"))?;

            if row >= lines.len() {
                // print welcome message if file is empty
            } else {
                let line = &lines[row].render;

                let line_len = cmp::min(
                    line.len().saturating_sub(offset.x as usize),
                    self.width as usize,
                );

                let start = if line_len == 0 { 0 } else { offset.x as usize };

                queue!(self.out, style::Print(&line[start..start + line_len]))?;
                queue!(self.out, style::Print(" "))?;
            }

            queue!(self.out, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    pub fn draw_status_bar(&mut self, buffer: &Buffer, cursor: &Position) -> io::Result<()> {
        let file_length = buffer.lines.len();

        let cur_line_len = if let Some(line) = buffer.get_line(cursor.y as usize) {
            line.render.len()
        } else {
            0
        };

        let file_info = format!(
            " {}|{}|{}|{} ",
            cursor.y + 1,
            file_length,
            cursor.x + 1,
            cur_line_len
        );

        queue!(self.out, style::SetAttribute(style::Attribute::Reverse))?;

        let file_path = if let Some(file) = &buffer.file_path {
            file.to_str().unwrap()
        } else {
            "NO PATH"
        };

        queue!(
            self.out,
            cursor::MoveTo(0, self.height),
            style::Print(file_path)
        )?;

        for i in file_path.len() as u16..self.status_bar.width {
            queue!(self.out, cursor::MoveTo(i, self.height))?;

            if self.width - i <= file_info.len() as u16 {
                queue!(self.out, style::Print(file_info))?;
                break;
            }

            queue!(self.out, style::Print(" "))?;
        }

        queue!(self.out, style::SetAttribute(style::Attribute::Reset))?;

        Ok(())
    }

    pub fn draw_message(&mut self, message: Option<&mut Message>) -> io::Result<()> {
        if message.is_none() {
            return Ok(());
        }

        let message = message.unwrap();

        // Maybe put this somewhere else so this function only draws
        if message.start_time.is_none() {
            message.start()
        }

        queue!(
            self.out,
            cursor::MoveTo(
                self.width - message.text.len() as u16,
                self.height - self.status_bar.height
            ),
            style::Print(&message.text)
        )?;

        Ok(())
    }

    pub fn draw_cursor(&mut self, pos: &Position, offset: &Position) -> io::Result<()> {
        queue!(
            self.out,
            cursor::Show,
            cursor::MoveTo(pos.x - offset.x + self.gutter_size as u16, pos.y - offset.y)
        )
    }
}
