use std::{io, time::Duration};

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Terminal Events
#[derive(Clone, Debug)]
pub enum Event {
    /// Tick Event
    Tick,
    /// Render
    Render,
    /// Initialization Event
    Init,
    /// Keyboard Event
    Key(KeyEvent),
    /// Mouse Event
    Mouse(MouseEvent),
    /// Terminal Resize Event
    Resize(u16, u16),
    /// Error Event
    Error,
    /// Exit Event
    Exit,
}

#[derive(Debug)]
pub struct EventHandler {
    tick_rate: f64,
    frame_time: f64,

    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,

    cancellation_token: CancellationToken,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: f64, frame_time: f64) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let handler = tokio::task::spawn(async move {});

        let cancellation_token = CancellationToken::new();

        Self {
            tick_rate,
            frame_time,

            cancellation_token,

            sender,
            receiver,
            handler,
        }
    }

    pub fn start(&mut self) {
        let tick_duration = Duration::from_secs_f64(1.0 / self.tick_rate);
        let frame_duration = Duration::from_secs_f64(1.0 / self.frame_time);

        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let _cancellation_token = self.cancellation_token.clone();

        let _sender = self.sender.clone();

        self.handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();

            let mut tick_interval = tokio::time::interval(tick_duration);
            let mut render_interval = tokio::time::interval(frame_duration);

            _sender.send(Event::Init).unwrap();

            loop {
                let tick_delay = tick_interval.tick();
                let frame_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = _cancellation_token.cancelled() => {
                        break;
                    }
                    _ = tick_delay => {
                        _sender.send(Event::Tick).unwrap();
                    }
                    _ = frame_delay => {
                        _sender.send(Event::Render).unwrap();
                    }
                    possible_event = crossterm_event => {
                        match possible_event {
                            Some(Ok(evt))  => {
                                match evt {
                                    CrosstermEvent::Key(key) => {
                                        if key.kind == crossterm::event::KeyEventKind::Press {
                                            _sender.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    CrosstermEvent::Mouse(mouse) => {
                                        _sender.send(Event::Mouse(mouse)).unwrap();
                                    },
                                    CrosstermEvent::Resize(x, y) => {
                                        _sender.send(Event::Resize(x, y)).unwrap();
                                    },
                                    CrosstermEvent::FocusLost => {
                                    },
                                    CrosstermEvent::FocusGained => {
                                    },
                                    CrosstermEvent::Paste(_) => {
                                    },
                                }
                            }
                            Some(Err(_)) => {
                                _sender.send(Event::Error).unwrap();

                            }
                            None => {},
                        }
                    }

                }
            }
        });
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn stop(&self) -> io::Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.handler.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.handler.abort();
            } else if counter > 100 {
                break;
            }
        }
        Ok(())
    }
}
