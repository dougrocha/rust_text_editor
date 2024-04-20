use crate::Rect;

#[derive(Clone)]
pub struct Cell {
    symbol: String,
}

impl Cell {
    pub fn set_cell(&mut self, character: &str) {
        self.symbol = character.to_string();
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: " ".to_string(),
        }
    }
}

pub struct Frame {
    pub cursor_position: Option<(u16, u16)>,

    pub size: Rect,
    pub cells: Vec<Cell>,
}

impl Frame {
    pub fn new(size: Rect) -> Self {
        let cells = vec![Cell::default(); size.area()];
        Self {
            cursor_position: Some((0, 0)),
            size,
            cells,
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, char: &str) {
        let index = y * self.size.width() + x;
        if let Some(cell) = self.cells.get_mut(index) {
            cell.set_cell(char);
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y * self.size.width() + x]
    }

    pub fn set_cursor_position(&mut self, x: u16, y: u16) {
        self.cursor_position = Some((x, y));
    }
}
