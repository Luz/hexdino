#[derive(PartialEq, Copy, Clone, Default)]
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
}
