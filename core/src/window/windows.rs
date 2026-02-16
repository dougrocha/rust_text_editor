use ratatui::layout::{Direction, Rect};

use crate::buffer::BufferId;

use super::layout_node::LayoutNode;
use super::window::{Window, WindowId};

#[derive(Default)]
pub struct Windows {
    root: Option<LayoutNode>,
    pub focused_window_id: Option<WindowId>,
    next_window_id: usize,

    /// total area for windows
    area: Rect,
}

impl Windows {
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            ..Default::default()
        }
    }

    /// Splits the target window in the given direction, creating a new window with the given buffer
    /// Returns the ID of the newly created window, which will be focused
    pub fn split(
        &mut self,
        target_id: WindowId,
        buffer_id: BufferId,
        split_dir: Direction,
    ) -> WindowId {
        let id = WindowId(self.next_window_id);
        self.next_window_id += 1;
        let new_window = Window::new(id, buffer_id, self.area);

        if let Some(root) = self.root.take() {
            self.root = Some(root.split_at(target_id, new_window, split_dir));
        }

        if let Some(root) = self.root.as_mut() {
            root.calculate_layout(self.area);
        }

        self.focus(id);
        id
    }

    /// Adds a new window to the layout showing the given buffer
    /// Returns the ID of the newly created window
    pub fn add_window(&mut self, buffer_id: BufferId) -> WindowId {
        let win_id = WindowId(self.next_window_id);
        self.next_window_id += 1;

        match self.root.take() {
            Some(root) => {
                let window = Window::new(win_id, buffer_id, self.area);

                self.root = Some(root.append(window, Direction::Vertical));
            }
            None => {
                self.root = Some(LayoutNode::Leaf {
                    window: Window::new(win_id, buffer_id, self.area),
                });
            }
        }

        if let Some(root) = self.root.as_mut() {
            root.calculate_layout(self.area);
        }

        win_id
    }

    /// Closes the window with the given ID
    /// Returns the buffer ID and window ID of the closed window, or None if not found
    pub fn close(&mut self, win_id: WindowId) -> Option<(BufferId, WindowId)> {
        let buf_id = self.iter().find(|win| win.id == win_id)?.buffer_id;

        let root_node = self.root.take()?;

        self.root = root_node.remove_window(win_id);

        let next_win = self.iter().next()?;
        self.focus(next_win.id);

        if let Some(root) = self.root.as_mut() {
            root.calculate_layout(self.area);
        }

        Some((buf_id, win_id))
    }

    pub fn focus(&mut self, id: WindowId) {
        self.focused_window_id = Some(id);

        for node in self.iter_mut() {
            node.focused = node.id == id;
        }
    }

    pub fn get_by_buffer_id(&self, buffer_id: BufferId) -> Vec<&Window> {
        self.iter()
            .filter(|window| window.buffer_id == buffer_id)
            .collect()
    }

    pub fn get_focused(&self) -> Option<&Window> {
        if let Some(focused_node) = self.focused_window_id {
            self.get(focused_node)
        } else {
            None
        }
    }

    pub fn get_focused_mut(&mut self) -> Option<&mut Window> {
        if let Some(focused_node) = self.focused_window_id {
            self.get_mut(focused_node)
        } else {
            None
        }
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn get(&self, win_id: WindowId) -> Option<&Window> {
        self.iter().find(|&window| window.id == win_id)
    }

    pub fn get_mut(&mut self, win_id: WindowId) -> Option<&mut Window> {
        self.iter_mut().find(|window| window.id == win_id)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn count(&self) -> usize {
        self.root.iter().len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Window> {
        self.root.as_ref().into_iter().flat_map(|node| node.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        self.root
            .as_mut()
            .into_iter()
            .flat_map(|node| node.iter_mut())
    }

    pub(crate) fn root(&self) -> Option<&LayoutNode> {
        self.root.as_ref()
    }
}
