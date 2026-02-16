use ratatui::layout::{Direction, Rect};

use super::iter::{WindowIter, WindowIterMut};
use super::window::{Window, WindowId};

/// A node in the window layout tree
#[derive(Debug)]
pub enum LayoutNode {
    /// A container that splits space between children
    Container {
        direction: Direction,
        children: Vec<LayoutNode>,
        /// Size ratios for each child (sum to 1.0)
        ratios: Option<Vec<f32>>,
    },
    /// A leaf node containing an actual window
    Leaf { window: Window },
}

impl LayoutNode {
    /// Creates a container with equal ratios for all children
    fn container_with_equal_ratios(direction: Direction, children: Vec<LayoutNode>) -> LayoutNode {
        let n = children.len();
        let ratios = Some(vec![1.0 / n as f32; n]);
        LayoutNode::Container {
            direction,
            children,
            ratios,
        }
    }

    /// Creates a container with 50/50 split
    fn container_split(direction: Direction, first: LayoutNode, second: LayoutNode) -> LayoutNode {
        LayoutNode::Container {
            direction,
            children: vec![first, second],
            ratios: Some(vec![0.5, 0.5]),
        }
    }

    /// Normalizes ratios to sum to 1.0
    fn normalize_ratios(ratios: &mut [f32]) {
        let sum: f32 = ratios.iter().sum();
        if sum > 0.0 {
            for r in ratios {
                *r /= sum;
            }
        }
    }

    /// Appends a new window to the layout tree in the given direction
    pub fn append(self, new_window: Window, dir: Direction) -> LayoutNode {
        match self {
            LayoutNode::Container {
                direction,
                mut children,
                ..
            } if direction == dir => {
                // Same direction: add to existing container
                children.push(LayoutNode::Leaf { window: new_window });
                Self::container_with_equal_ratios(direction, children)
            }
            LayoutNode::Container {
                direction,
                children,
                ratios,
            } => {
                // Different direction: wrap existing container
                let existing = LayoutNode::Container {
                    direction,
                    children,
                    ratios,
                };
                Self::container_split(dir, existing, LayoutNode::Leaf { window: new_window })
            }
            LayoutNode::Leaf { window } => {
                // Split leaf into container
                Self::container_split(
                    dir,
                    LayoutNode::Leaf { window },
                    LayoutNode::Leaf { window: new_window },
                )
            }
        }
    }

    /// Splits the window with the given ID by inserting a new window adjacent to it
    pub fn split_at(self, target_id: WindowId, new_window: Window, dir: Direction) -> LayoutNode {
        match self {
            // Target is a direct child and direction matches
            LayoutNode::Container {
                direction,
                mut children,
                ratios,
            } if direction == dir => {
                if let Some(idx) = Self::find_target_child_index(&children, target_id) {
                    // Insert new window after target and recalculate ratios
                    children.insert(idx + 1, LayoutNode::Leaf { window: new_window });
                    Self::container_with_equal_ratios(direction, children)
                } else {
                    // Target is deeper in tree, recursively search
                    let children = children
                        .into_iter()
                        .map(|child| child.split_at(target_id, new_window, dir))
                        .collect();
                    LayoutNode::Container {
                        direction,
                        children,
                        ratios,
                    }
                }
            }
            // Direction doesn't match, recursively search children
            LayoutNode::Container {
                direction,
                children,
                ratios,
            } => {
                let children = children
                    .into_iter()
                    .map(|child| child.split_at(target_id, new_window, dir))
                    .collect();
                LayoutNode::Container {
                    direction,
                    children,
                    ratios,
                }
            }
            // Found target leaf, split it
            LayoutNode::Leaf { window } if window.id == target_id => Self::container_split(
                dir,
                LayoutNode::Leaf { window },
                LayoutNode::Leaf { window: new_window },
            ),
            // Not the target
            other => other,
        }
    }

    /// Finds the index of a direct child leaf with the given window ID
    fn find_target_child_index(children: &[LayoutNode], target_id: WindowId) -> Option<usize> {
        children
            .iter()
            .position(|c| matches!(c, LayoutNode::Leaf { window } if window.id == target_id))
    }

    /// Removes the window with the given ID from the tree, returning None if tree becomes empty
    pub fn remove_window(self, target_id: WindowId) -> Option<LayoutNode> {
        match self {
            LayoutNode::Container {
                direction,
                children,
                ratios,
            } => {
                // Recursively remove from children, keeping track of ratios
                let (new_children, new_ratios) =
                    Self::remove_from_children(children, ratios, target_id);

                // If all children removed, this container is empty
                if new_children.is_empty() {
                    return None;
                }

                // If only one child left, unwrap the container
                if new_children.len() == 1 {
                    return Some(new_children.into_iter().next().unwrap());
                }

                // Flatten nested containers with same direction
                let (flat_children, mut flat_ratios) =
                    Self::flatten_children(new_children, new_ratios, direction);

                // Normalize ratios to sum to 1.0
                Self::normalize_ratios(&mut flat_ratios);

                Some(LayoutNode::Container {
                    direction,
                    children: flat_children,
                    ratios: Some(flat_ratios),
                })
            }
            LayoutNode::Leaf { window } => {
                if target_id == window.id {
                    None
                } else {
                    Some(LayoutNode::Leaf { window })
                }
            }
        }
    }

    /// Recursively removes target window from children, preserving ratios
    fn remove_from_children(
        children: Vec<LayoutNode>,
        ratios: Option<Vec<f32>>,
        target_id: WindowId,
    ) -> (Vec<LayoutNode>, Vec<f32>) {
        let mut new_children = Vec::with_capacity(children.len());
        let mut new_ratios = Vec::with_capacity(ratios.as_ref().map_or(0, |r| r.len()));

        for (i, child) in children.into_iter().enumerate() {
            if let Some(kept_child) = child.remove_window(target_id) {
                new_children.push(kept_child);
                if let Some(ref r) = ratios {
                    if let Some(&ratio) = r.get(i) {
                        new_ratios.push(ratio);
                    }
                }
            }
        }

        (new_children, new_ratios)
    }

    /// Flattens nested containers with the same direction into a single level
    fn flatten_children(
        children: Vec<LayoutNode>,
        ratios: Vec<f32>,
        parent_direction: Direction,
    ) -> (Vec<LayoutNode>, Vec<f32>) {
        let mut flat_children = Vec::new();
        let mut flat_ratios = Vec::new();
        let default_ratio = 1.0 / children.len() as f32;

        for (i, child) in children.into_iter().enumerate() {
            let parent_ratio = ratios.get(i).copied().unwrap_or(default_ratio);

            match child {
                LayoutNode::Container {
                    direction: child_dir,
                    children: grandchildren,
                    ratios: grandchild_ratios,
                } if child_dir == parent_direction => {
                    // Same direction: flatten into parent
                    for (j, grandchild) in grandchildren.into_iter().enumerate() {
                        let grandchild_ratio = grandchild_ratios
                            .as_ref()
                            .and_then(|r| r.get(j).copied())
                            .unwrap_or(1.0);
                        flat_children.push(grandchild);
                        flat_ratios.push(parent_ratio * grandchild_ratio);
                    }
                }
                other => {
                    // Different direction or leaf: keep as is
                    flat_children.push(other);
                    flat_ratios.push(parent_ratio);
                }
            }
        }

        (flat_children, flat_ratios)
    }

    pub fn iter(&self) -> WindowIter<'_> {
        WindowIter { stack: vec![self] }
    }

    pub fn iter_mut(&mut self) -> WindowIterMut<'_> {
        WindowIterMut { stack: vec![self] }
    }

    /// Counts the number of splits in the given direction (used for grid-based layout)
    pub fn count_splits(&self, dir: Direction) -> usize {
        match self {
            LayoutNode::Leaf { .. } => 1,
            LayoutNode::Container {
                direction,
                children,
                ..
            } => {
                if *direction == dir {
                    children.iter().map(|c| c.count_splits(dir)).sum()
                } else {
                    children
                        .iter()
                        .map(|c| c.count_splits(dir))
                        .max()
                        .unwrap_or(1)
                }
            }
        }
    }

    /// Calculates and assigns layout rectangles to all windows in the tree
    pub fn calculate_layout(&mut self, available_space: Rect) {
        // Calculate the grid dimensions based on maximum splits in each direction
        let total_cols = self.count_splits(Direction::Vertical);
        let total_rows = self.count_splits(Direction::Horizontal);

        let col_width = available_space.width / total_cols as u16;
        let row_height = available_space.height / total_rows as u16;

        self.assign_layout(available_space, col_width, row_height);
    }

    /// Recursively assigns layout rectangles to children based on their split counts
    fn assign_layout(&mut self, space: Rect, col_width: u16, row_height: u16) {
        match self {
            LayoutNode::Container {
                direction,
                children,
                ..
            } => {
                let mut offset = 0u16;

                for child in children.iter_mut() {
                    // Each child gets space proportional to its split count
                    let child_splits = child.count_splits(*direction);

                    let child_rect = match direction {
                        Direction::Vertical => Rect {
                            x: space.x + offset,
                            y: space.y,
                            width: col_width * child_splits as u16,
                            height: space.height,
                        },
                        Direction::Horizontal => Rect {
                            x: space.x,
                            y: space.y + offset,
                            width: space.width,
                            height: row_height * child_splits as u16,
                        },
                    };

                    offset += match direction {
                        Direction::Vertical => child_rect.width,
                        Direction::Horizontal => child_rect.height,
                    };

                    child.assign_layout(child_rect, col_width, row_height);
                }
            }
            LayoutNode::Leaf { window } => {
                window.area = space;
            }
        }
    }

    /// Finds the container node that directly contains the window with the given ID
    pub fn find_window_container_mut(&mut self, target_id: WindowId) -> Option<&mut LayoutNode> {
        let is_target_container = match self {
            LayoutNode::Container { children, .. } => children.iter().any(
                |child| matches!(child, LayoutNode::Leaf { window } if window.id == target_id),
            ),
            LayoutNode::Leaf { .. } => false,
        };

        if is_target_container {
            return Some(self);
        }

        if let LayoutNode::Container { children, .. } = self {
            for child in children.iter_mut() {
                if let Some(found_container) = child.find_window_container_mut(target_id) {
                    return Some(found_container);
                }
            }
        }

        None
    }
}
