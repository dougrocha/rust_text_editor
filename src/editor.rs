use std::{fs::File, io::BufReader, path::Path};

use color_eyre::eyre::Result;
use ratatui::layout::Rect;
use ropey::Rope;

use crate::{
    buffer::{BufferId, Buffers},
    components::{self, Component, EventPropagation, Position},
    mode::Mode,
    terminal::Event,
    window::Windows,
};

pub struct Editor {
    area: Rect,
    pub mode: Mode,
    pub needs_redraw: bool,
    pub buffers: Buffers,
    pub windows: Windows,
    should_quit: bool,
}

impl Editor {
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            mode: Mode::Normal,
            needs_redraw: false,
            windows: Windows::new(area),
            buffers: Buffers::new(),
            should_quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn open(&mut self, file_path: &Path) -> Result<BufferId> {
        if let Some(buffer_id) = self.buffers.find_by_file_path(file_path) {
            let window_id = self.windows.add(buffer_id);
            self.windows.focus(window_id);

            return Ok(buffer_id);
        }

        let content = Rope::from_reader(BufReader::new(File::open(file_path)?))?;

        let buffer_id = self.buffers.add(content, Some(file_path));
        let window_id = self.windows.add(buffer_id);
        self.windows.focus(window_id);

        Ok(buffer_id)
    }
}

#[derive(Default)]
pub struct EditorView {}

impl EditorView {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for EditorView {
    fn handle_events(
        &mut self,
        event: &Event,
        context: &mut components::Context,
    ) -> EventPropagation {
        match event {
            Event::Key(key_event) => {}
            Event::Mouse(mouse_event) => {}
            _ => {}
        }

        EventPropagation::Ignore(None)
    }

    fn cursor(&self, area: Rect, context: &mut Editor) -> Option<Position> {
        if let Some(window) = context.windows.get_focused() {
            let buffer = context.buffers.get(window.buffer_id).unwrap();
            Some(
                buffer
                    .get_cursor(window.id)
                    .unwrap()
                    .to_screen_position(buffer.content()),
            )
        } else {
            None
        }
    }

    fn render(
        &self,
        f: &mut crate::terminal::Frame<'_>,
        area: Rect,
        context: &mut crate::components::Context,
    ) {
        todo!()
    }
}
