use super::{LayoutNode, Window};

pub struct WindowIter<'a> {
    pub stack: Vec<&'a LayoutNode>,
}

impl<'a> Iterator for WindowIter<'a> {
    type Item = &'a Window;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(cur) = self.stack.pop() {
            match cur {
                LayoutNode::Container {
                    direction: _,
                    children,
                    ratios: _,
                } => {
                    for child in children.iter().rev() {
                        self.stack.push(child);
                    }
                }
                LayoutNode::Leaf { window } => return Some(window),
            }
        }

        None
    }
}

pub struct WindowIterMut<'a> {
    pub stack: Vec<&'a mut LayoutNode>,
}

impl<'a> Iterator for WindowIterMut<'a> {
    type Item = &'a mut Window;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(cur) = self.stack.pop() {
            match cur {
                LayoutNode::Container { children, .. } => {
                    // Notice we use iter_mut() here so we get mutable references
                    // to the children to push onto our mutable stack.
                    for child in children.iter_mut().rev() {
                        self.stack.push(child);
                    }
                }
                LayoutNode::Leaf { window } => {
                    return Some(window);
                }
            }
        }
        None
    }
}
