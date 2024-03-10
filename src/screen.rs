use crossterm::{cursor, queue, style, terminal};
use std::io::{self, Write};

pub struct Screen {
    out: io::Stdout,

    width: u16,
    height: u16,
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
            cursor::MoveTo(0, 0)
        )
    }

    pub fn refresh_screen(&mut self) -> io::Result<()> {
        self.clear()?;
        self.draw_rows()?;

        self.flush()
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }

    fn draw_rows(&mut self) -> io::Result<()> {
        // let lines = &self.rows.lines;

        for row in 0..self.height {
            queue!(io::stdout(), cursor::MoveTo(0, row))?;
            queue!(io::stdout(), style::Print("~"))?;

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
            queue!(
                io::stdout(),
                terminal::Clear(terminal::ClearType::UntilNewLine)
            )?;
        }

        Ok(())
    }

    pub fn cursor_position(&self) -> io::Result<(u16, u16)> {
        cursor::position()
    }
}
