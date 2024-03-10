pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Cursor {
    cx: usize,
    cy: usize,
    // pos_offset: (usize, usize),
    // config: Rc<Config>,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            cx: 0,
            cy: 0,
            // pos_offset: (0, 0),
            // config,
        }
    }

    pub fn move_cursor(
        &mut self,
        direction: Direction, // , file_manager: &Rows
    ) {
        // let lines = &file_manager.lines;
        let lines = [0; 10];

        match direction {
            Direction::Up => {
                self.cy = self.cy.saturating_sub(1);
            }
            Direction::Down => {
                if self.cy < lines.len().saturating_sub(1) {
                    self.cy += 1;
                }
            }
            Direction::Left => {
                self.cx = self.cx.saturating_sub(1);
            }
            Direction::Right => {
                // if self.cx < lines[self.cy].len().saturating_sub(1) {
                //     self.cx += 1;
                // }
                self.cx += 1;
            }
        }

        // if self.cx > lines[self.cy].len() {
        //     self.cx = lines[self.cy].len().saturating_sub(1);
        // }
    }

    // fn scroll(&mut self) {
    // self.pos_offset.1 = cmp::min(self.pos_offset.1, self.cy);
    // if self.cy >= self.pos_offset.1 + self.screen.1 {
    //     self.pos_offset.1 = self.cy - self.screen.1 + 1;
    // }
    //
    // self.pos_offset.0 = cmp::min(self.pos_offset.0, self.cx);
    // if self.cx >= self.pos_offset.0 + self.screen.0 {
    //     self.pos_offset.0 = self.cx - self.screen.0 + 1;
    // }
    // }
}
