#[derive(Debug)]
pub enum Action {
    /// Tick Event
    Tick,
    /// Render Event
    Render,
    /// Initialization Event
    Init,
    /// Terminal Resize Event
    Resize(u16, u16),
    /// Keyboard Event
    Key(crossterm::event::KeyEvent),
    /// Quit Event
    Quit,
}
