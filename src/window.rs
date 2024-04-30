use crate::buffer::BufferId;

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
        for (idx, node) in self.nodes.iter().enumerate() {
            if node.id == id {
                self.focused_node = idx;
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
}

impl Window {
    pub fn new(id: VisibleBufferId) -> Self {
        Self { id }
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
}
