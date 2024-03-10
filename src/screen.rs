use crossterm::{cursor, queue, style, terminal};
use std::{
    cmp,
    io::{self, Write},
};

use crate::editor::Line;

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

pub struct Screen {
    pub out: io::Stdout,

    pub width: u16,
    pub height: u16,
}

impl Screen {
    pub fn new() -> Self {
        let (columns, rows) = terminal::size().unwrap();
        Self {
            out: io::stdout(),
            width: columns,
            height: rows,
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
            // queue!(io::stdout(), style::Print("~"))?;

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

            // if i + self.cursor.pos_offset.1 >= lines.len() {
            //     break;
            // }
            //
            // let line = &lines[i + self.cursor.pos_offset.1].content;
            // let col_offet = self.cursor.pos_offset.0;
            //
            // let line_len = cmp::min(
            //     line.len().saturating_sub(self.cursor.pos_offset.0),
            //     self.screen.width,
            // );
            // let start = if line_len == 0 { 0 } else { col_offet };
            //
            // let line = line[start..start + line_len].to_string();
            //
            // let line = if self.config.relative_number {
            //     let line_number = i + self.cursor.pos_offset.1 + 1;
            //
            //     let line_number = if line_number == self.cursor.cy + 1 {
            //         line_number
            //     } else {
            //         match line_number > self.cursor.cy + 1 {
            //             true => line_number - self.cursor.cy - 1,
            //             false => self.cursor.cy - line_number + 1,
            //         }
            //     };
            //
            //     format!("{:>4} {}", line_number, line)
            // } else {
            //     line
            // };
            //
            // queue!(io::stdout(), style::Print(line),)?;
            //
            queue!(self.out, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    pub fn draw_cursor(&mut self, pos: &Position, offset: &Position) -> io::Result<()> {
        queue!(
            self.out,
            cursor::Show,
            cursor::MoveTo(pos.x - offset.x, pos.y - offset.y)
        )
    }
}
