use crate::{
    action::{Action, BufferAction, BuffersAction, CursorAction},
    buffer::BufferId,
};

#[derive(Default)]
pub struct Windows {
    pub nodes: Vec<Window>,
    focused_node: usize,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            focused_node: 0,
        }
    }
    pub fn add(&mut self, id: VisibleBufferId) {
        self.nodes.push(Window::new(id));
    }

    pub fn focus(&mut self, id: VisibleBufferId) {
        for (idx, node) in self.nodes.iter_mut().enumerate() {
            if node.id == id {
                self.focused_node = idx;
                node.focused = true;
            }
        }
    }

    pub fn get_focused(&self) -> Option<&Window> {
        self.nodes.get(self.focused_node)
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

pub struct Window {
    pub id: VisibleBufferId,
    pub focused: bool,
}

impl Window {
    pub fn new(id: VisibleBufferId) -> Self {
        Self { id, focused: false }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CursorId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleBufferId {
    pub buffer_id: BufferId,
    pub cursor_id: CursorId,
}

impl VisibleBufferId {
    pub fn new(buffer_id: BufferId, cursor_id: CursorId) -> Self {
        Self {
            buffer_id,
            cursor_id,
        }
    }

    fn send_buffer_action(&self, action: BufferAction) -> Action {
        Action::Buffer(BuffersAction {
            buffer_id: self.buffer_id,
            inner_action: action,
        })
    }

    fn send_cursor_action(&self, action: CursorAction) -> Action {
        self.send_buffer_action(BufferAction::CursorAction {
            cursor_id: self.cursor_id,
            action,
        })
    }

    pub fn insert_char(&self, char: char) -> Action {
        self.send_cursor_action(CursorAction::InsertChar(char))
    }

    pub fn insert_newline(&self) -> Action {
        self.send_cursor_action(CursorAction::InsertNewLine)
    }

    pub fn start_of_line(&self) -> Action {
        self.send_cursor_action(CursorAction::StartOfLine)
    }

    pub fn end_of_line(&self) -> Action {
        self.send_cursor_action(CursorAction::EndOfLine)
    }

    pub fn move_left(&self, n: usize) -> Action {
        self.send_cursor_action(CursorAction::Left(n))
    }

    pub fn move_right(&self, n: usize) -> Action {
        self.send_cursor_action(CursorAction::Right(n))
    }

    pub fn move_up(&self, n: usize) -> Action {
        self.send_cursor_action(CursorAction::Up(n))
    }

    pub fn move_down(&self, n: usize) -> Action {
        self.send_cursor_action(CursorAction::Down(n))
    }
}
