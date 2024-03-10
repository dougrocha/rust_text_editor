use std::io;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};

use crate::{
    cursor::{Cursor, Direction},
    keyboard::Keyboard,
    screen::Screen,
};

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,

    cursor: Cursor,
    // rows: Rows,
    // config: Rc<Config>,
}

impl Editor {
    pub fn new() -> Self {
        // let config = Rc::new(Config::new());

        Self {
            screen: Screen::new(),
            keyboard: Keyboard,
            cursor: Cursor::new(),
            // rows: Rows::new(),
            // config,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        loop {
            if self.screen.refresh_screen().is_err() {
                self.die("Error refreshing screen");
            }

            if self.process_keypress()? {
                break;
            }
        }

        // loop {
        //
        //     self.cursor_controller.scroll();
        //     self.clear_screen()?;
        //
        //     self.draw_rows()?;
        //
        //     queue!(
        //         stdout,
        //         cursor::Show,
        //         cursor::MoveTo(
        //             self.config.gutter_size as u16 + self.cursor_controller.cx as u16
        //                 - self.cursor_controller.pos_offset.0 as u16,
        //             self.cursor_controller.cy as u16
        //                 - self.cursor_controller.pos_offset.1 as u16
        //         )
        //     )?;
        //
        //     stdout.flush()?;
        // }

        terminal::disable_raw_mode()
    }

    fn die<S: Into<String>>(&mut self, msg: S) {
        self.screen.clear().unwrap();
        terminal::disable_raw_mode().unwrap();
        eprintln!("{}", msg.into());
        std::process::exit(1);
    }

    /// Process the keypress and return whether the editor should exit
    fn process_keypress(&mut self) -> io::Result<bool> {
        match self.keyboard.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                return Ok(true);
            }
            // KeyEvent {
            //     code:
            //         key_code @ KeyCode::Up
            //         | key_code @ KeyCode::Down
            //         | key_code @ KeyCode::Left
            //         | key_code @ KeyCode::Right,
            //     modifiers: KeyModifiers::NONE,
            //     kind: KeyEventKind::Press,
            //     ..
            // } => {
            //     self.screen.move_cursor(key_code)?;
            // }
            _ => {}
        };

        Ok(false)
    }
}
