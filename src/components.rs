use ratatui::layout::Rect;

use crate::{
    editor::Editor,
    terminal::{Event, Frame},
};

pub struct Components {
    components: Vec<Box<dyn Component>>,
    area: Rect,
}

impl Components {
    pub fn new(area: Rect) -> Self {
        Self {
            components: vec![],
            area,
        }
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn resize(&mut self, area: Rect) {
        self.area = area;
    }

    pub fn push(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    pub fn cursor(&self, frame: &mut Frame<'_>, context: &mut Context) -> Option<Position> {
        for component in self.components.iter().rev() {
            if let Some(cursor) = component.cursor(frame.size(), context.editor) {
                return Some(cursor);
            }
        }

        None
    }

    /// Handle terminal events and return should_redraw
    pub fn handle_events(&mut self, event: &Event, context: &mut Context) -> bool {
        let mut callbacks = vec![];
        let mut stop_propagation = false;

        for component in self.components.iter_mut().rev() {
            // if event is not handled
            // it will propagate upwards
            match component.handle_events(event, context) {
                EventPropagation::Ignore(Some(cb)) => {
                    callbacks.push(cb);
                }
                EventPropagation::Consume(Some(cb)) => {
                    callbacks.push(cb);
                    stop_propagation = true;
                    break;
                }
                EventPropagation::Ignore(None) => {}
                EventPropagation::Consume(None) => {
                    stop_propagation = true;
                }
            }
        }

        for cb in callbacks {
            cb(self, context);
        }

        stop_propagation
    }

    /// Render from bottom up in components stack
    pub fn render(&mut self, frame: &mut Frame<'_>, context: &mut Context) {
        for component in &mut self.components {
            component.render(frame, frame.size(), context);
        }
    }
}

pub struct Context<'a> {
    pub editor: &'a mut Editor,
}

/// Event callback to be called when event is either done propagating
/// or has passed through all components
type EventCallback = Box<dyn FnOnce(&mut Components, &mut Context)>;

pub enum EventPropagation {
    Ignore(Option<EventCallback>),
    Consume(Option<EventCallback>),
}

pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// `Component` is a trait that represents a visual and interactive element of the user interface.
/// Implementors of this trait can be registered with the main application loop and will be able to receive events,
/// update state, and be rendered on the screen.
pub trait Component {
    /// Handle events for current component
    ///
    /// Returns whether or not the event was consumed by the component
    fn handle_events(&mut self, _event: &Event, _context: &mut Context) -> EventPropagation {
        EventPropagation::Ignore(None)
    }

    fn cursor(&self, _area: Rect, _context: &mut Editor) -> Option<Position> {
        None
    }
    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    /// * `context` - References to editor.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn render(&self, f: &mut Frame<'_>, area: Rect, context: &mut Context);
}
