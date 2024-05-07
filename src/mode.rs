use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Search,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Visual => write!(f, "VISUAL"),
            Mode::Search => write!(f, "SEARCH"),
        }
    }
}
