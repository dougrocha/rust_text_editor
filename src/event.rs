use std::time::Duration;

use crossterm::event;
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

/// Terminal Events
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Tick Event
    Tick,
    /// Keyboard Event
    Key(event::KeyEvent),
    /// Mouse Event
    Mouse(event::MouseEvent),
    /// Terminal Resize Event
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = _sender.closed() => {
                    break;
                  }
                  _ = tick_delay => {
                    _sender.send(Event::Tick).unwrap();
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    match evt {
                      event::Event::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                          _sender.send(Event::Key(key)).unwrap();
                        }
                      },
                      event::Event::Mouse(mouse) => {
                        _sender.send(Event::Mouse(mouse)).unwrap();
                      },
                      event::Event::Resize(x, y) => {
                        _sender.send(Event::Resize(x, y)).unwrap();
                      },
                      event::Event::FocusLost => {
                      },
                      event::Event::FocusGained => {
                      },
                      event::Event::Paste(_) => {
                      },
                    }
                  }
                };
            }
        });
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> crate::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "This is an IO error",
            )))
    }
}
