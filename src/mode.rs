pub enum Mode {
    Normal,
    Insert,
    Command,
    Visual,
    VisualLine,
}

impl Mode {
    fn is_insert(&self) -> bool {
        matches!(self, Mode::Insert)
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, Mode::Normal)
    }

    fn is_command(&self) -> bool {
        matches!(self, Mode::Command)
    }

    fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual)
    }

    fn is_visual_line(&self) -> bool {
        matches!(self, Mode::VisualLine)
    }
}
