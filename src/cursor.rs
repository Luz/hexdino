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
    pub fn selects(&self) -> CursorSelects {
        self.sel
    }
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
    pub fn swap_selection_hex_ascii(&mut self) {
        if self.is_over_ascii() {
            self.select_left_nibble();
        } else {
            self.select_ascii();
        }
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
        if self.is_over_right_nibble() {
            self.select_left_nibble();
        }
    }
    pub fn jump_to_end_of_line(&mut self, columns: usize, upperlimit: usize) {
        self.pos = self.calculate_end_of_line(columns);
        if self.is_over_left_nibble() {
            self.select_right_nibble();
        }
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
            remaining = remaining.saturating_sub(2 * (pos_after.saturating_sub(pos_before)));

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
            if self.is_over_right_nibble() && self.pos < upperlimit.saturating_sub(1) {
                self.pos += 1;
                self.select_left_nibble();
                // No return as function ends here anyway
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
            remaining = remaining.saturating_sub(2 * (pos_before.saturating_sub(pos_after)));

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
            if self.is_over_left_nibble() && self.pos > 0 {
                self.pos -= 1;
                self.select_right_nibble();
                // No return as function ends here anyway
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
    pub fn move_to_line(&mut self, line: usize, columns: usize, upperlimit: usize) {
        let pos_on_line = self.calculate_pos_on_line(columns);
        self.jump_to_pos_on_line(line, pos_on_line, columns, upperlimit);
    }
}

#[cfg(test)]
#[path = "./cursor_test.rs"]
mod cursor_test;
