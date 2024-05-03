use std::{fs::File, io::BufReader, path::PathBuf};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    text::{Line, Text},
};
use ropey::Rope;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::{
    action::{Action, BufferAction, BuffersAction, CursorAction},
    buffer::Buffers,
    components::Component,
    config::Config,
    mode::Mode,
    tui,
    utils::version,
    window::{CursorId, VisibleBufferId, Windows},
};

#[derive(Clone)]
pub struct Context {
    pub action_tx: Option<UnboundedSender<Action>>,
    pub current_working_directory: PathBuf,
    pub file_paths: Vec<PathBuf>,
    pub config: Config,
    pub mode: Mode,
}

pub struct Editor {
    context: Context,
    should_quit: bool,
    buffers: Buffers,
    windows: Windows,
}

impl Editor {
    pub fn new(cwd: PathBuf, file_paths: Vec<PathBuf>) -> Result<Self> {
        let context = Context {
            action_tx: None,
            config: Config::new()?,
            current_working_directory: cwd,
            file_paths,
            mode: Mode::Normal,
        };

        Ok(Self {
            context,
            should_quit: false,
            buffers: Buffers::new(),
            windows: Windows::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new()?;
        // tui.mouse(true);
        tui.enter()?;

        self.init(tui.size()?, action_tx.clone())?;

        loop {
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) if key.code == KeyCode::Char('q') => {
                        action_tx.send(Action::Quit)?;
                    }
                    _ => {}
                }

                if let Some(action) = self.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?
                };
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Quit => self.should_quit = true,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            let r = self.draw(f, f.size());
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            let r = self.draw(f, f.size());
                            if let Err(e) = r {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                    .unwrap();
                            }
                        })?;
                    }
                    _ => {}
                }

                if let Some(action) = self.update(action.clone())? {
                    action_tx.send(action)?
                };
            }

            if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}

impl Component for Editor {
    fn init(&mut self, _area: Rect, action_tx: UnboundedSender<Action>) -> Result<()> {
        for file_path in self.context.file_paths.iter().cloned() {
            action_tx.send(Action::OpenFile(file_path))?;
        }

        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let mut event = None;

        let window = self.windows.get_focused().unwrap();

        event = match self.context.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('k') => Some(Action::Buffer(BuffersAction {
                    buffer_id: window.id.buffer_id,
                    inner_action: BufferAction::CursorAction {
                        cursor_id: window.id.cursor_id,
                        action: CursorAction::Up(1),
                    },
                })),
                KeyCode::Char('j') => Some(Action::Buffer(BuffersAction {
                    buffer_id: window.id.buffer_id,
                    inner_action: BufferAction::CursorAction {
                        cursor_id: window.id.cursor_id,
                        action: CursorAction::Down(1),
                    },
                })),
                KeyCode::Char('h') => Some(Action::Buffer(BuffersAction {
                    buffer_id: window.id.buffer_id,
                    inner_action: BufferAction::CursorAction {
                        cursor_id: window.id.cursor_id,
                        action: CursorAction::Left(1),
                    },
                })),
                KeyCode::Char('l') => Some(Action::Buffer(BuffersAction {
                    buffer_id: window.id.buffer_id,
                    inner_action: BufferAction::CursorAction {
                        cursor_id: window.id.cursor_id,
                        action: CursorAction::Right(1),
                    },
                })),
                KeyCode::Char('i') => {
                    self.context.mode = Mode::Insert;
                    None
                }
                _ => None,
            },
            Mode::Insert => match key.code {
                KeyCode::Char(c) => Some(Action::Buffer(BuffersAction {
                    buffer_id: window.id.buffer_id,
                    inner_action: BufferAction::CursorAction {
                        cursor_id: window.id.cursor_id,
                        action: CursorAction::InsertChar(c),
                    },
                })),
                KeyCode::Esc => {
                    self.context.mode = Mode::Normal;
                    None
                }
                _ => None,
            },
            Mode::Visual => todo!(),
            Mode::Search => todo!(),
        };

        Ok(event)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::OpenFile(file_path) => {
                if let Some(buffer_id) = self.buffers.find_by_file_path(&file_path) {
                    let visible_buffer_id = VisibleBufferId::new(buffer_id, CursorId::default());
                    self.windows.focus(visible_buffer_id);

                    return Ok(None);
                }

                let content = Rope::from_reader(BufReader::new(File::open(&file_path)?))?;

                let buffer_id = self.buffers.add(content, Some(file_path));

                let visible_buffer_id = VisibleBufferId::new(buffer_id, CursorId::default());
                self.windows.add(visible_buffer_id);
                self.windows.focus(visible_buffer_id);
            }
            Action::Buffer(buffer_action) => {
                self.buffers.handle_actions(buffer_action);
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw(&self, f: &mut tui::Frame<'_>, area: Rect) -> Result<()> {
        if self.windows.is_empty() {
            let version = version();

            let lines: Vec<Line> = version.lines().map(Line::from).collect();

            f.render_widget(Text::from(lines), area);
        } else {
            for window in &self.windows.nodes {
                let visible_buffer_id = window.id;
                let buffer_id = visible_buffer_id.buffer_id;

                let buffer = self.buffers.get(buffer_id);

                if let Some(buffer) = buffer {
                    buffer.draw(f, area, visible_buffer_id.cursor_id, &self.context)?;
                    let cursor = buffer.get_cursor(visible_buffer_id.cursor_id);

                    let x = buffer
                        .content
                        .line_to_char(buffer.content.char_to_line(cursor.range.start));
                    let y = buffer.content.char_to_line(cursor.range.start);
                    f.set_cursor(x as u16, y as u16);
                }
            }
        }

        Ok(())
    }
}
