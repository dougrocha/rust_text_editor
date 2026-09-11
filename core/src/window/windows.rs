use ratatui::layout::{Direction, Rect};

use crate::buffer::BufferId;

use super::layout_node::LayoutNode;
use super::window::{Window, WindowId};

/// A spatial direction for moving window focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDir {
    Left,
    Right,
    Up,
    Down,
}

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

        // Copy the offset from the target window to the new window
        let target_offset = self.get(target_id).map(|w| w.offset).unwrap_or_default();
        let mut new_window = Window::new(id, buffer_id, self.area);
        new_window.offset = target_offset;

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
        // Find the window to close and its position in iteration order
        let windows: Vec<WindowId> = self.iter().map(|w| w.id).collect();
        let closed_position = windows.iter().position(|&id| id == win_id)?;
        let buf_id = self.get(win_id)?.buffer_id;

        let root_node = self.root.take()?;
        self.root = root_node.remove_window(win_id);

        // Focus the previous window, or first if closed was first
        let window_to_focus = if closed_position > 0 {
            // Focus the window that was before the closed one
            windows.get(closed_position - 1).copied()
        } else {
            // Closed window was first, focus the new first window
            self.iter().next().map(|w| w.id)
        };

        if let Some(focus_id) = window_to_focus {
            self.focus(focus_id);
        }

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

    /// Moves focus to the nearest window in `dir`; returns `true` if focus moved.
    pub fn focus_direction(&mut self, dir: FocusDir) -> bool {
        let Some(current) = self.get_focused().map(|w| (w.id, w.area)) else {
            return false;
        };

        let candidates: Vec<(WindowId, Rect)> = self
            .iter()
            .filter(|w| w.id != current.0)
            .map(|w| (w.id, w.area))
            .collect();

        match best_neighbor(current.1, &candidates, dir) {
            Some(target) => {
                self.focus(target);
                true
            }
            None => false,
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

    /// Updates the total area and re-tiles every window to fit it.
    pub fn resize(&mut self, area: Rect) {
        self.area = area;
        if let Some(root) = self.root.as_mut() {
            root.calculate_layout(area);
        }
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

/// Picks the closest window fully on the `dir` side of `current`, preferring axis-aligned ones.
fn best_neighbor(current: Rect, candidates: &[(WindowId, Rect)], dir: FocusDir) -> Option<WindowId> {
    let cur_mid_x = current.x as i32 + current.width as i32 / 2;
    let cur_mid_y = current.y as i32 + current.height as i32 / 2;

    let mut best: Option<(WindowId, i32)> = None;
    for &(id, area) in candidates {
        let on_side = match dir {
            FocusDir::Left => area.x + area.width <= current.x,
            FocusDir::Right => area.x >= current.x + current.width,
            FocusDir::Up => area.y + area.height <= current.y,
            FocusDir::Down => area.y >= current.y + current.height,
        };
        if !on_side {
            continue;
        }

        let mid_x = area.x as i32 + area.width as i32 / 2;
        let mid_y = area.y as i32 + area.height as i32 / 2;
        let (along, perp) = match dir {
            FocusDir::Left | FocusDir::Right => ((mid_x - cur_mid_x).abs(), (mid_y - cur_mid_y).abs()),
            FocusDir::Up | FocusDir::Down => ((mid_y - cur_mid_y).abs(), (mid_x - cur_mid_x).abs()),
        };
        let score = along + perp * 3;

        if best.map_or(true, |(_, best_score)| score < best_score) {
            best = Some((id, score));
        }
    }

    best.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(id: usize, area: Rect) -> (WindowId, Rect) {
        (WindowId(id), area)
    }

    #[test]
    fn moves_across_a_two_by_two_grid() {
        // windows 0,1,2,3 arranged as a 2x2 grid, top-left to bottom-right
        let top_left = Rect::new(0, 0, 50, 20);
        let grid = [
            win(0, top_left),
            win(1, Rect::new(50, 0, 50, 20)),
            win(2, Rect::new(0, 20, 50, 20)),
            win(3, Rect::new(50, 20, 50, 20)),
        ];

        assert_eq!(best_neighbor(top_left, &grid[1..], FocusDir::Right), Some(WindowId(1)));
        assert_eq!(best_neighbor(top_left, &grid[1..], FocusDir::Down), Some(WindowId(2)));
        assert_eq!(best_neighbor(top_left, &grid[1..], FocusDir::Left), None);
        assert_eq!(best_neighbor(top_left, &grid[1..], FocusDir::Up), None);
    }

    #[test]
    fn picks_the_aligned_pane_when_several_are_on_that_side() {
        let current = Rect::new(0, 20, 50, 20);
        let candidates = [
            win(1, Rect::new(50, 0, 50, 20)),  // right, misaligned up
            win(2, Rect::new(50, 20, 50, 20)), // right, aligned
            win(3, Rect::new(50, 40, 50, 20)), // right, misaligned down
        ];
        assert_eq!(best_neighbor(current, &candidates, FocusDir::Right), Some(WindowId(2)));
    }
}
