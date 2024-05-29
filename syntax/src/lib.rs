// https://tree-sitter.github.io/tree-sitter/syntax-highlighting
// https://www.youtube.com/watch?v=rHgXmG8U5YQ&list=PL9KpW-9Hl_het1V3_dLhG_0K99a9043ac&index=6

use std::ops::Range;

use ratatui::style::Color;
use ropey::RopeSlice;
use tree_sitter::{Node, Parser, Query, QueryCursor, TextProvider, Tree};

const CANCELLATION_CHECK_INTERVAL: usize = 100;

#[derive(Debug)]
enum Error {
    Cancelled,
    InvalidLanguage,
    Unknown,
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
    query: Query,
}

impl Highlight {
    pub fn new(content: RopeSlice) -> Self {
        let query = Query::new(
            &tree_sitter_rust::language(),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
        )
        .unwrap();

        let mut highlight = Self {
            parser: Parser::new(),
            root: None,
            query,
        };

        highlight.update(content);

        highlight
    }

    pub fn tree(&self) -> &Tree {
        self.root.as_ref().unwrap()
    }

    fn update(&mut self, content: RopeSlice) {
        let content = content.slice(..);

        match &self.root {
            Some(_tree) => {}
            None => {
                self.parse(content).unwrap();
            }
        }
    }

    fn parse(&mut self, content: RopeSlice) -> Result<(), Error> {
        let parser = &mut self.parser;

        parser.set_timeout_micros(1000 * 500);
        parser
            .set_language(&tree_sitter_rust::language())
            .map_err(|_| Error::InvalidLanguage)?;

        let tree = parser
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
        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(range);

        let captures = cursor.captures(&self.query, self.tree().root_node(), RopeProvider(content));

        let mut colors = Vec::new();

        for (match_, capture_index) in captures {
            let capture = match_.captures[capture_index];

            let range = capture.node.byte_range();

            let color = match self.query.capture_names()[capture.index as usize] {
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
        let highlighter = Highlight::new(content);

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
}
