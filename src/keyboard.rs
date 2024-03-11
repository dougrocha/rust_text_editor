use std::{io, time};

use crossterm::event::{self};

pub struct Keyboard;

impl Keyboard {
    pub fn read_key(&self) -> io::Result<event::KeyEvent> {
        loop {
            if event::poll(time::Duration::from_millis(100))? {
                if let event::Event::Key(key_event) = event::read()? {
                    return Ok(key_event);
                }
            } else {
                return Ok(event::KeyEvent::from(event::KeyCode::Null));
            }
        }
    }
}
