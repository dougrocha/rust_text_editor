mod cursor;
mod keyboard;
mod screen;

mod editor;
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
//
// struct Rows {
//     lines: Vec<Line>,
//     file_path: Option<PathBuf>,
// }
//
// impl Rows {
//     fn new() -> Self {
//         let mut args = env::args();
//
//         match args.nth(1) {
//             Some(file) => Rows::from(file.as_ref()),
//             None => Self {
//                 lines: Vec::new(),
//                 file_path: None,
//             },
//         }
//     }
// }
//
// impl From<&Path> for Rows {
//     fn from(file: &Path) -> Self {
//         let file_contents = fs::read_to_string(file).expect("Unable to read file");
//         Self {
//             file_path: Some(file.to_path_buf()),
//             lines: file_contents.lines().map(|s| Line::new(s.into())).collect(),
//         }
//     }
// }
//
// struct Line {
//     content: Box<str>,
//     rendered: String,
// }
//
// impl Line {
//     fn new(content: Box<str>) -> Self {
//         Self {
//             content,
//             rendered: String::new(),
//         }
//     }
// }
