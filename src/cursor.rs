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
    pub fn get_current_line(&self, columns: usize) -> usize {
        self.pos / columns
    }
    pub fn get_last_line(&self, columns: usize, upperlimit: usize) -> usize {
        // upperlimit is supposedly vec.len() but we want the last element,
        // so we subtract 1. To ensure no "-1" for the usize, we saturate
        let last_element: usize = upperlimit.saturating_sub(1);
        last_element / columns
    }
    #[allow(dead_code)]
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
    pub fn move_n_right(&mut self, amount: usize, upperlimit: usize) {
        if self.is_over_ascii() {
            self.add(amount, upperlimit);
            self.trim_to_max_minus_one(upperlimit);
        } else {
            let mut remaining = amount;

            let pos_before = self.pos;
            self.add(remaining / 2, upperlimit);
            let pos_after = self.pos;
            remaining -= 2 * (pos_after - pos_before);

            if remaining == 0 {
                return;
            }
            if self.is_over_left_nibble() {
                self.select_right_nibble();
                remaining = remaining.saturating_sub(1);
            }
            if remaining == 0 {
                return;
            }
            if self.is_over_right_nibble() {
                if self.pos < upperlimit.saturating_sub(1) {
                    self.pos += 1;
                    self.select_left_nibble();
                    return;
                }
            }
        }
    }
    pub fn move_n_left(&mut self, amount: usize) {
        if self.is_over_ascii() {
            self.sub(amount, 0);
        } else {
            let mut remaining = amount;

            let pos_before = self.pos;
            self.sub(remaining / 2, 0);
            let pos_after = self.pos;
            remaining -= 2 * (pos_before - pos_after);

            if remaining == 0 {
                return;
            }
            if self.is_over_right_nibble() {
                self.select_left_nibble();
                remaining = remaining.saturating_sub(1);
            }
            if remaining == 0 {
                return;
            }
            if self.is_over_left_nibble() {
                if self.pos > 0 {
                    self.pos -= 1;
                    self.select_right_nibble();
                    return;
                }
            }
        }
    }
    pub fn move_n_down(&mut self, amount: usize, columns: usize, upperlimit: usize) {
        let pos_on_line = self.calculate_pos_on_line(columns);
        let mut newline = amount + self.get_current_line(columns);
        let lastline = self.get_last_line(columns, upperlimit);
        if newline > lastline {
            newline = lastline;
        }
        self.jump_to_pos_on_line(newline, pos_on_line, columns, upperlimit);
    }
    pub fn move_n_up(&mut self, amount: usize, columns: usize, upperlimit: usize) {
        let pos_on_line = self.calculate_pos_on_line(columns);
        self.sub(amount * columns, 0);
        let currentline = self.get_current_line(columns);
        self.jump_to_pos_on_line(currentline, pos_on_line, columns, upperlimit);
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
#[test]
fn cursor_ascii_move_0_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::AsciiChar;
    cursor.move_n_right(0, buf.len());
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_ascii_move_1_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::AsciiChar;
    cursor.move_n_right(1, buf.len());
    assert_eq!(cursor.pos, 1);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_ascii_move_2_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::AsciiChar;
    cursor.move_n_right(2, buf.len());
    assert_eq!(cursor.pos, 2);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_ascii_move_3_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::AsciiChar;
    cursor.move_n_right(3, buf.len());
    assert_eq!(cursor.pos, 2);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_left_nibble_move_0_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_right(0, buf.len());
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_right_nibble_move_0_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_right(0, buf.len());
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_left_nibble_move_1_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_right(1, buf.len());
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_right_nibble_move_1_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_right(1, buf.len());
    assert_eq!(cursor.pos, 1);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_left_nibble_move_2_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_right(2, buf.len());
    assert_eq!(cursor.pos, 1);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_right_nibble_move_2_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_right(2, buf.len());
    assert_eq!(cursor.pos, 1);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_left_nibble_move_4_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_right(4, buf.len());
    assert_eq!(cursor.pos, 2);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_right_nibble_move_4_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_right(4, buf.len());
    assert_eq!(cursor.pos, 2);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_left_nibble_move_5_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_right(5, buf.len());
    assert_eq!(cursor.pos, 2);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_right_nibble_move_5_right() {
    // Create data from 0 to 2 as test data
    let buf: Vec<u8> = (0..3).collect();
    // 0x00 0x01 0x02
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_right(5, buf.len());
    assert_eq!(cursor.pos, 2);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_ascii_move_0_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::AsciiChar;
    cursor.move_n_left(0);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
    cursor.pos = 1;
    cursor.move_n_left(0);
    assert_eq!(cursor.pos, 1);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_ascii_move_1_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::AsciiChar;
    cursor.move_n_left(1);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
    cursor.pos = 1;
    cursor.move_n_left(1);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_ascii_move_2_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::AsciiChar;
    cursor.move_n_left(2);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
    cursor.pos = 1;
    cursor.move_n_left(2);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_any_nibble_move_0_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_left(0);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_left(0);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
    cursor.pos = 1;
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_left(0);
    assert_eq!(cursor.pos, 1);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
    cursor.pos = 1;
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_left(0);
    assert_eq!(cursor.pos, 1);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_left_nibble_move_1_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_left(1);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
    cursor.pos = 1;
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_left(1);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_right_nibble_move_1_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_left(1);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
    cursor.pos = 1;
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_left(1);
    assert_eq!(cursor.pos, 1);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_left_nibble_move_2_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_left(2);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
    cursor.pos = 1;
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_left(2);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_right_nibble_move_2_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_left(2);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
    cursor.pos = 1;
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_left(2);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
}
#[test]
fn cursor_left_nibble_move_3_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_left(3);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
    cursor.pos = 1;
    cursor.sel = CursorSelects::LeftNibble;
    cursor.move_n_left(3);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_right_nibble_move_3_left() {
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_left(3);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
    cursor.pos = 1;
    cursor.sel = CursorSelects::RightNibble;
    cursor.move_n_left(3);
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_get_current_line() {
    let mut cursor = Cursor::default();
    assert_eq!(cursor.get_current_line(1), 0);
    cursor.pos = 1;
    assert_eq!(cursor.get_current_line(1), 1);
    cursor.pos = 2;
    assert_eq!(cursor.get_current_line(1), 2);
    cursor.pos = 65535;
    assert_eq!(cursor.get_current_line(1), 65535);
    for pos in 0..=15 {
        cursor.pos = pos;
        assert_eq!(cursor.get_current_line(16), 0);
    }
    for pos in 16..=31 {
        cursor.pos = pos;
        assert_eq!(cursor.get_current_line(16), 1);
    }
    for cols in 2..=20 {
        cursor.pos = 0;
        assert_eq!(cursor.get_current_line(cols), 0);
        cursor.pos = 1;
        assert_eq!(cursor.get_current_line(cols), 0);
        cursor.pos = cols - 1;
        assert_eq!(cursor.get_current_line(cols), 0);
        cursor.pos = cols;
        assert_eq!(cursor.get_current_line(cols), 1);
        cursor.pos = 16 * cols - 1;
        assert_eq!(cursor.get_current_line(cols), 15);
        cursor.pos = 16 * cols;
        assert_eq!(cursor.get_current_line(cols), 16);
        cursor.pos = 16 * cols + 1;
        assert_eq!(cursor.get_current_line(cols), 16);
        cursor.pos = 256 * cols - 1;
        assert_eq!(cursor.get_current_line(cols), 256 - 1);
        cursor.pos = 256 * cols;
        assert_eq!(cursor.get_current_line(cols), 256);
        cursor.pos = 4096 * cols - 1;
        assert_eq!(cursor.get_current_line(cols), 4096 - 1);
        cursor.pos = 4096 * cols;
        assert_eq!(cursor.get_current_line(cols), 4096);
        cursor.pos = 4096 * cols + 1;
        assert_eq!(cursor.get_current_line(cols), 4096);
    }
}
