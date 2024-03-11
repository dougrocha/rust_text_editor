mod buffer;
mod editor;
mod keyboard;
mod screen;
mod status_message;

use editor::Editor;

fn main() -> std::io::Result<()> {
    let mut app = Editor::new();

    app.start()?;

    Ok(())
}

// struct Config {
//     relative_number: bool,
//
//     gutter_size: usize,
// }
//
// impl Config {
//     fn new() -> Self {
//         Self {
//             relative_number: true,
//
//             gutter_size: 5,
//         }
//     }
// }
