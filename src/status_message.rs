use std::time;

#[derive(Debug, Clone)]
pub struct Message {
    pub text: String,
    pub duration: time::Duration,

    pub start_time: Option<time::Instant>,
}

impl Message {
    /// Create a new message with a default duration of 5 seconds.
    pub fn new(text: String) -> Self {
        Message::with_duration(text, time::Duration::from_secs(5))
    }

    /// Create a new message with a custom duration.
    pub fn with_duration(text: String, duration: time::Duration) -> Self {
        Self {
            text,
            duration,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(time::Instant::now());
    }

    pub fn has_expired(&self) -> bool {
        if let Some(start_time) = self.start_time {
            start_time.elapsed() > self.duration
        } else {
            false
        }
    }
}
