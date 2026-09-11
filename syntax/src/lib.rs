// https://tree-sitter.github.io/tree-sitter/syntax-highlighting
// https://www.youtube.com/watch?v=rHgXmG8U5YQ&list=PL9KpW-9Hl_het1V3_dLhG_0K99a9043ac&index=6

use std::{ops::Range, path::Path};

use ratatui::style::Color;
use ropey::RopeSlice;
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Query, QueryCursor, TextProvider, Tree};

#[derive(Debug)]
enum Error {
    Cancelled,
    InvalidLanguage,
    _Unknown,
}

/// Picks the grammar + highlight query for a file extension; `None` means "no highlighting".
fn language_for_extension(extension: &str) -> Option<(Language, &'static str)> {
    match extension {
        "rs" => Some((tree_sitter_rust::language(), tree_sitter_rust::HIGHLIGHTS_QUERY)),
        _ => None,
    }
}

// Adapter to convert rope chunks to bytes
pub struct ChunksBytes<'a> {
    chunks: ropey::iter::Chunks<'a>,
}
impl<'a> Iterator for ChunksBytes<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(str::as_bytes)
    }
}

pub struct RopeProvider<'a>(pub RopeSlice<'a>);
impl<'a> TextProvider<&'a [u8]> for RopeProvider<'a> {
    type I = ChunksBytes<'a>;

    fn text(&mut self, node: Node) -> Self::I {
        let fragment = self.0.byte_slice(node.start_byte()..node.end_byte());
        ChunksBytes {
            chunks: fragment.chunks(),
        }
    }
}

#[derive(Debug)]
pub struct HighlightInfo {
    pub range: Range<usize>,
    pub color: Color,
}

pub struct Highlight {
    parser: Parser,
    root: Option<Tree>,
    /// `None` when the file's extension has no known grammar - highlighting is a no-op then.
    language: Option<Language>,
    query: Option<Query>,
}

impl Highlight {
    /// `path` picks the grammar by extension; unrecognized or missing extensions get no highlighting.
    pub fn new(content: RopeSlice, path: Option<&Path>) -> Self {
        let grammar = path
            .and_then(Path::extension)
            .and_then(|ext| ext.to_str())
            .and_then(language_for_extension);

        let (language, query) = match grammar {
            Some((language, query_src)) => {
                let query = Query::new(&language, query_src).unwrap();
                (Some(language), Some(query))
            }
            None => (None, None),
        };

        let mut highlight = Self {
            parser: Parser::new(),
            root: None,
            language,
            query,
        };

        highlight.update(content, None);

        highlight
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.root.as_ref()
    }

    pub fn update(&mut self, content: RopeSlice, edit: Option<&InputEdit>) {
        if self.language.is_none() {
            return;
        }

        let content = content.slice(..);

        if let (Some(tree), Some(input_edit)) = (self.root.as_mut(), edit) {
            tree.edit(input_edit);
        }

        self.parse(content).unwrap()
    }

    fn parse(&mut self, content: RopeSlice) -> Result<(), Error> {
        let Some(language) = self.language.as_ref() else {
            return Ok(());
        };

        self.parser.set_timeout_micros(1000 * 500);
        self.parser
            .set_language(language)
            .map_err(|_| Error::InvalidLanguage)?;

        let tree = self
            .parser
            .parse_with(
                &mut |byte, _| {
                    if byte <= content.len_bytes() {
                        let (chunk, start_byte, _, _) = content.chunk_at_byte(byte);
                        &chunk.as_bytes()[byte - start_byte..]
                    } else {
                        // out of range
                        &[]
                    }
                },
                self.root.as_ref(),
            )
            .ok_or(Error::Cancelled)?;

        self.root = Some(tree);
        Ok(())
    }

    pub fn colors(&self, content: RopeSlice, range: Range<usize>) -> Vec<HighlightInfo> {
        let (Some(query), Some(tree)) = (&self.query, self.tree()) else {
            return Vec::new();
        };

        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(range);

        let captures = cursor.captures(query, tree.root_node(), RopeProvider(content));

        let mut colors = Vec::new();

        for (match_, capture_index) in captures {
            let capture = match_.captures[capture_index];

            let range = capture.node.byte_range();

            let color = match query.capture_names()[capture.index as usize] {
                "keyword" => Color::Blue,
                "string" => Color::Green,
                "constructor" => Color::Rgb(156, 207, 216),
                "function" => Color::Rgb(182, 147, 148),
                "punctuation.bracket" => Color::DarkGray,
                "punctuation.delimiter" => Color::Gray,
                "property" => Color::LightGreen,
                "type" => Color::LightGreen,
                "comment" => Color::Green,
                _ => Color::Gray,
            };

            colors.push(HighlightInfo { range, color })
        }

        colors
    }
}

pub fn byte_to_point(content: &RopeSlice, byte_offset: usize) -> Point {
    let char_offset = content.byte_to_char(byte_offset);
    let line = content.char_to_line(char_offset);
    let line_start_char = content.line_to_char(line);
    let column = char_offset - line_start_char;

    Point { row: line, column }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style},
    };
    use ropey::Rope;

    use std::{fs::File, io::BufReader};

    #[test]
    fn it_works() {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::language();

        parser
            .set_language(&language)
            .expect("Error loading Rust grammar");

        let file_path = "../core/src/editor.rs";
        //let file_path = "../test.rs";
        let content = Rope::from_reader(BufReader::new(File::open(file_path).unwrap())).unwrap();

        let content = content.slice(..);
        let highlighter = Highlight::new(content, Some(std::path::Path::new(file_path)));

        let vertical = 1;
        let height = 49;
        let range = {
            let last_line = content.len_lines().saturating_sub(1);
            let last_visible_line = (vertical + height as usize)
                .saturating_sub(1)
                .min(last_line);

            let start = content.line_to_byte(vertical.min(last_line));
            let end = content.line_to_byte(last_visible_line + 1);
            start..end
        };
        let colors = highlighter.colors(content.slice(..), range.clone());

        println!("range: {:?}", range);
        println!("line: {:?}", content.line(1));
        println!(
            "content byte len: {:?}",
            content.byte_slice(range.start..range.end).to_string()
        );

        println!("color ranges: {:#?}", colors);

        let mut buf = Buffer::empty(Rect {
            height: 50,
            width: 100,
            x: 0,
            y: 0,
        });
        let start = range.start;
        let end = range.end;

        let mut x: u16 = 0;
        let mut y: u16 = 0;
        let mut style = Style::default();

        for (index, char) in content.slice(start..end).chars().enumerate() {
            if char == '\n' {
                y += 1;
                x = 0;
                continue;
            }

            if let Some(c) = colors.iter().find(|x| x.range.contains(&(index + start))) {
                style = style.fg(c.color);
            } else {
                style = style.fg(Color::White);
            }

            buf.set_string(x, y, char.to_string(), style);

            x += 1;
        }
    }

    #[test]
    fn rust_files_get_highlighted() {
        let content = Rope::from_str("let x = 1;\n");
        let content = content.slice(..);
        let rs = Highlight::new(content, Some(std::path::Path::new("main.rs")));
        assert!(!rs.colors(content, 0..content.len_bytes()).is_empty());
    }

    #[test]
    fn non_rust_files_get_no_highlighting() {
        let content = Rope::from_str("This is plain prose, not rust code.\n");
        let content = content.slice(..);

        let txt = Highlight::new(content, Some(std::path::Path::new("notes.txt")));
        assert!(txt.colors(content, 0..content.len_bytes()).is_empty());

        let no_ext = Highlight::new(content, Some(std::path::Path::new("README")));
        assert!(no_ext.colors(content, 0..content.len_bytes()).is_empty());

        let no_path = Highlight::new(content, None);
        assert!(no_path.colors(content, 0..content.len_bytes()).is_empty());
    }
}
