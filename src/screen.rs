use crossterm::{cursor, queue, style, terminal};
use std::{
    cmp,
    io::{self, Write},
};

use crate::{
    buffer::{Buffer, Line},
    editor::Messages,
    mode::Mode,
};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Default, Clone, Copy)]
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

    pub gutter_size: u16,

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

            gutter_size: 4,

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

            queue!(self.out, cursor::MoveTo(self.gutter_size, i))?;

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

    pub fn draw_gutter(
        &mut self,
        lines: &[Line],
        cursor: &Position,
        offset: &Position,
    ) -> io::Result<()> {
        let line_numbers = true;
        let relative_numbers = true;

        for i in 0..self.height {
            queue!(self.out, cursor::MoveTo(0, i))?;

            let row = i as usize + offset.y as usize;

            if row >= lines.len() {
                continue;
            }

            if line_numbers {
                let line_number = if relative_numbers {
                    if cursor.y == row as u16 {
                        row + 1
                    } else {
                        (row as i32 - cursor.y as i32).unsigned_abs() as usize
                    }
                } else {
                    row + 1
                };

                queue!(
                    self.out,
                    style::Print(format!(
                        "{:>width$} ",
                        line_number,
                        width = self.gutter_size as usize - 1
                    ))
                )?;
            }
        }
        Ok(())
    }

    pub fn draw_status_bar(
        &mut self,
        buffer: &Buffer,
        cursor: &Position,
        mode: &Mode,
    ) -> io::Result<()> {
        let file_length = buffer.lines.len();

        let cur_line_len = if let Some(line) = buffer.get_line(cursor.y as usize) {
            line.render.len()
        } else {
            0
        };

        queue!(self.out, style::SetAttribute(style::Attribute::Reverse))?;

        let cmd = match mode {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Visual => "Visual",
            Mode::VisualLine => "V-Line",
            Mode::Command => "Command",
        };
        let cmd = format!(" {} ", cmd);

        let file_path = if let Some(file) = &buffer.file_path {
            file.to_str().unwrap()
        } else {
            "NO PATH"
        };
        let file_path = format!(" {} ", file_path);
        let file_info = format!(
            " {}|{}|{}|{} ",
            cursor.y + 1,
            file_length,
            cursor.x + 1,
            cur_line_len
        );

        let start_len = file_path.len() + cmd.len();

        queue!(self.out, cursor::MoveTo(0, self.height), style::Print(cmd))?;
        queue!(self.out, style::Print(file_path))?;

        for i in start_len as u16..self.status_bar.width {
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

    pub fn draw_message(&mut self, messages: &mut Messages) -> io::Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        for (i, msg) in messages.iter_mut().enumerate() {
            if msg.start_time.is_none() {
                msg.start()
            }

            let start_msg_height = self.height - 1 - i as u16;

            queue!(
                self.out,
                cursor::MoveTo(self.width - msg.text.len() as u16, start_msg_height),
                style::Print(&msg.text)
            )?;
        }

        Ok(())
    }

    pub fn draw_cursor(&mut self, pos: &Position, offset: &Position) -> io::Result<()> {
        queue!(
            self.out,
            cursor::Show,
            cursor::MoveTo(pos.x - offset.x + self.gutter_size, pos.y - offset.y)
        )
    }
}
