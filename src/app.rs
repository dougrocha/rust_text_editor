use color_eyre::eyre::Result;
use ratatui::layout::Rect;

use crate::{
    cli::Args,
    components::{Components, Context},
    cursor::Cursor,
    editor::{Editor, EditorView},
    terminal::{Event, Terminal},
};

pub struct App {
    components: Components,
    terminal: Terminal,
    pub editor: Editor,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let terminal = Terminal::new()?;

        let area = terminal.size()?;

        let mut components = Components::new(area);

        let mut editor = Editor::new(area);

        let editor_view = Box::new(EditorView::new());
        components.push(editor_view);

        for path in args.files {
            let id = editor.open(&path).unwrap();

            let window_id = editor.windows.get_focused().unwrap().id;
            let buf = editor.buffers.get_mut(id).unwrap();
            buf.set_cursor(window_id, Cursor::default());
        }

        Ok(Self {
            components,
            terminal,
            editor,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.terminal.enter()?;

        self.event_loop().await;

        self.terminal.exit()?;

        Ok(())
    }

    async fn event_loop(&mut self) {
        self.terminal.start();

        self.render_app().await;

        loop {
            if self.editor.should_quit() {
                break;
            }

            tokio::select! {
                Some(term_event) = self.terminal.next() => {
                    self.handle_terminal_events(term_event).await;
                }
            }
        }
    }

    /// Handle possible events from the terminal
    async fn handle_terminal_events(&mut self, event: Event) {
        let mut context = Context {
            editor: &mut self.editor,
        };

        let redraw = match event {
            Event::Resize(width, height) => {
                self.terminal
                    .resize(Rect::new(0, 0, width, height))
                    .expect("Not able to resize terminal");

                let new_area = self.terminal.size().expect("Not able to get terminal size");

                self.components.resize(new_area);
                self.components.handle_events(
                    &Event::Resize(new_area.width, new_area.height),
                    &mut context,
                );

                true
            }
            event => self.components.handle_events(&event, &mut context),
        };

        // handle redraw event only if editor will not quit soon
        if redraw && !self.editor.should_quit() {
            self.render_app().await;
        }
    }

    async fn render_app(&mut self) {
        let mut frame_context = Context {
            editor: &mut self.editor,
        };

        frame_context.editor.needs_redraw = false;

        let _ = self.terminal.draw(|frame| {
            self.components.render(frame, &mut frame_context);
            if let Some(position) = self.components.cursor(frame, &mut frame_context) {
                frame.set_cursor(position.x as u16, position.y as u16);
            }
        });
    }

    // pub async fn run(&mut self) -> Result<()> {
    //     let (action_tx, mut action_rx) = mpsc::unbounded_channel();
    //
    //     let mut tui = tui::Tui::new()?;
    //     // tui.mouse(true);
    //     tui.enter()?;
    //
    //     self.init(tui.size()?, action_tx.clone())?;
    //
    //     loop {
    //         if let Some(e) = tui.next().await {
    //             match e {
    //                 tui::Event::Quit => action_tx.send(Action::Quit)?,
    //                 tui::Event::Tick => action_tx.send(Action::Tick)?,
    //                 tui::Event::Render => action_tx.send(Action::Render)?,
    //                 tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
    //                 tui::Event::Key(key) if key.code == KeyCode::Char('q') => {
    //                     action_tx.send(Action::Quit)?;
    //                 }
    //                 _ => {}
    //             }
    //
    //             if let Some(action) = self.handle_events(Some(e.clone()))? {
    //                 action_tx.send(action)?
    //             };
    //         }
    //
    //         while let Ok(action) = action_rx.try_recv() {
    //             if action != Action::Tick && action != Action::Render {
    //                 log::debug!("{action:?}");
    //             }
    //             match action {
    //                 Action::Quit => self.should_quit = true,
    //                 Action::Resize(w, h) => {
    //                     tui.resize(Rect::new(0, 0, w, h))?;
    //                     tui.draw(|f| {
    //                         let r = self.draw(f, f.size());
    //                         if let Err(e) = r {
    //                             action_tx
    //                                 .send(Action::Error(format!("Failed to draw: {:?}", e)))
    //                                 .unwrap();
    //                         }
    //                     })?;
    //                 }
    //                 Action::Render => {
    //                     tui.draw(|f| {
    //                         let r = self.draw(f, f.size());
    //                         if let Err(e) = r {
    //                             action_tx
    //                                 .send(Action::Error(format!("Failed to draw: {:?}", e)))
    //                                 .unwrap();
    //                         }
    //                     })?;
    //                 }
    //                 _ => {}
    //             }
    //
    //             if let Some(action) = self.update(action.clone())? {
    //                 action_tx.send(action)?
    //             };
    //         }
    //
    //         if self.should_quit {
    //             tui.stop()?;
    //             break;
    //         }
    //     }
    //     tui.exit()?;
    //     Ok(())
    // }
}

// impl Component for Editor {
//     fn init(&mut self, _area: Rect, action_tx: UnboundedSender<Action>) -> Result<()> {
//         for file_path in self.context.file_paths.iter().cloned() {
//             action_tx.send(Action::OpenFile(file_path))?;
//         }
//
//         Ok(())
//     }
//
//     fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
//         let window = self.windows.get_focused_mut().unwrap();
//
//         let event = match self.context.mode {
//             Mode::Normal => match key.code {
//                 // KeyCode::Char('k') => Some(window.move_up(1)),
//                 // KeyCode::Char('j') => Some(window.move_down(1)),
//                 // KeyCode::Char('h') => Some(window.move_left(1)),
//                 // KeyCode::Char('l') => Some(window.move_right(1)),
//                 // KeyCode::Char('$') => Some(window.end_of_line()),
//                 // KeyCode::Char('0') => Some(window.start_of_line()),
//                 // KeyCode::Char('i') => {
//                 //     self.context.mode = Mode::Insert;
//                 //     None
//                 // }
//                 _ => None,
//             },
//             Mode::Insert => match key.code {
//                 // KeyCode::Char(c) => Some(window.insert_char(c)),
//                 // KeyCode::Enter => Some(window.insert_newline()),
//                 // KeyCode::Esc => {
//                 //     self.context.mode = Mode::Normal;
//                 //     None
//                 // }
//                 _ => None,
//             },
//             Mode::Visual => todo!(),
//             Mode::Search => todo!(),
//         };
//
//         let buffer = self.buffers.get(window.buffer_id).unwrap();
//         window.ensure_cursor_in_view(&buffer.content, 8);
//
//         Ok(event)
//     }
//
//     fn update(&mut self, action: Action) -> Result<Option<Action>> {
//         match action {
//             Action::OpenFile(file_path) => {
//             }
//             // Action::Buffer(buffer_action) => {
//             //     self.buffers.handle_actions(buffer_action);
//             // }
//             _ => {}
//         }
//
//         Ok(None)
//     }
//
//     fn draw(&self, f: &mut tui::Frame<'_>, area: Rect) -> Result<()> {
//         if self.windows.is_empty() {
//             let version = version();
//
//             let lines: Vec<Line> = version.lines().map(Line::from).collect();
//
//             f.render_widget(Text::from(lines), area);
//         } else {
//             for window in &self.windows.nodes {
//                 let buffer_id = window.buffer_id;
//
//                 let buffer = self.buffers.get(buffer_id);
//
//                 if let Some(buffer) = buffer {
//                     buffer.draw(f, &self.context, &window)?;
//                     // let cursor = buffer
//                     //     .get_cursor(visible_buffer_id.cursor_id)
//                     //     .to_screen_position(&buffer.content);
//                     //
//                     // f.set_cursor(cursor.0, cursor.1);
//                 }
//             }
//         }
//
//         Ok(())
//     }
// }
