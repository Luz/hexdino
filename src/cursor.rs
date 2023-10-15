#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub enum CursorSelects {
    #[default]
    LeftNibble,
    RightNibble,
    AsciiChar,
}

#[derive(Copy, Clone, Default)]
pub struct Cursor {
    pos: usize,
    sel: CursorSelects,
}

impl Cursor {
    pub fn is_over_left_nibble(&self) -> bool {
        self.sel == CursorSelects::LeftNibble
    }
    pub fn is_over_right_nibble(&self) -> bool {
        self.sel == CursorSelects::RightNibble
    }
    pub fn is_over_ascii(&self) -> bool {
        self.sel == CursorSelects::AsciiChar
    }
    pub fn select_left_nibble(&mut self) {
        self.sel = CursorSelects::LeftNibble;
    }
    pub fn select_right_nibble(&mut self) {
        self.sel = CursorSelects::RightNibble;
    }
    pub fn select_ascii(&mut self) {
        self.sel = CursorSelects::AsciiChar;
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn set_pos(&mut self, new_position: usize) {
        self.pos = new_position;
    }
    pub fn trim_to_max_minus_one(&mut self, upperlimit: usize) {
        // upperlimit is supposedly vec.len() but we want the last element,
        // so we subtract 1. To ensure no "-1" for the usize, we saturate
        let last_element: usize = upperlimit.saturating_sub(1);
        if self.pos > last_element {
            // Trim to range
            self.pos = last_element;
        }
    }
    pub fn trim_to_min(&mut self, lowerlimit: usize) {
        // lowerlimit is mostly just 0, but we can set it to anything else
        if self.pos < lowerlimit {
            self.pos = lowerlimit;
        }
    }
    pub fn add(&mut self, addend: usize, upperlimit: usize) {
        self.pos += addend;
        self.trim_to_max_minus_one(upperlimit);
    }
    pub fn sub(&mut self, subtrahend: usize, lowerlimit: usize) {
        self.pos = self.pos.saturating_sub(subtrahend);
        self.trim_to_min(lowerlimit);
    }
    pub fn calculate_pos_on_line(&self, columns: usize) -> usize {
        self.pos % columns
    }
    pub fn calculate_start_of_line(&self, columns: usize) -> usize {
        self.pos - self.calculate_pos_on_line(columns)
    }
    pub fn calculate_end_of_line(&self, columns: usize) -> usize {
        self.calculate_start_of_line(columns) + (columns - 1)
    }
    pub fn jump_to_start_of_line(&mut self, columns: usize) {
        self.pos = self.calculate_start_of_line(columns);
    }
    pub fn jump_to_end_of_line(&mut self, columns: usize, upperlimit: usize) {
        self.pos = self.calculate_end_of_line(columns);
        self.trim_to_max_minus_one(upperlimit);
    }
    pub fn jump_to_line(&mut self, line: usize, columns: usize, upperlimit: usize) {
        self.set_pos(line * columns);
        self.trim_to_max_minus_one(upperlimit);
    }
    pub fn jump_to_pos_on_line(
        &mut self,
        line: usize,
        pos_on_line: usize,
        columns: usize,
        upperlimit: usize,
    ) {
        self.set_pos(line * columns + pos_on_line);
        self.trim_to_max_minus_one(upperlimit);
    }
}

#[test]
fn cursor_default() {
    let cursor = Cursor::default();
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_jump_to_start_of_line() {
    // Assuming the column width is COLS
    const COLS: usize = 16;
    // Create data from 0 to 19 as test data
    let buf: Vec<u8> = (0..20).collect();
    // Cursor by defaults points to the first element
    let mut cursor = Cursor::default();
    // Select the last element
    cursor.pos = buf.len().saturating_sub(1);
    assert_eq!(cursor.pos, 19);
    cursor.jump_to_start_of_line(COLS);
    assert_eq!(cursor.pos, 16);
    // Go to the last element of the first line
    cursor.sub(1, 0);
    cursor.jump_to_start_of_line(COLS);
    assert_eq!(cursor.pos, 0);
}
#[test]
fn cursor_jump_to_end_of_line() {
    // Assuming the column width is COLS
    const COLS: usize = 16;
    // Create data from 0 to 19 as test data
    let buf: Vec<u8> = (0..20).collect();
    // Cursor by defaults points to the first element
    let mut cursor = Cursor::default();
    // Since the line is COLS wide, end of line is 15
    cursor.jump_to_end_of_line(COLS, buf.len());
    assert_eq!(cursor.pos, 15);
    // Go to the first element of the next line
    cursor.add(1, buf.len());
    println!("{:?}", buf.len());
    cursor.jump_to_end_of_line(COLS, buf.len());
    assert_eq!(cursor.pos, 19);
}
