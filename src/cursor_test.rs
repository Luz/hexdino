use super::*;

#[test]
fn cursor_default() {
    let cursor = Cursor::default();
    assert_eq!(cursor.pos, 0);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_jump_to_start_of_line_ensure_position() {
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
fn cursor_jump_to_start_of_line_selects_left_nibble() {
    const COLS: usize = 16;
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.jump_to_start_of_line(COLS);
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
#[test]
fn cursor_jump_to_end_of_line_ensure_position() {
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
fn cursor_jump_to_end_of_line_selects_right_nibble() {
    const COLS: usize = 16;
    // Create data from 0 to 3 as test data
    let buf: Vec<u8> = (0..2).collect();
    let mut cursor = Cursor::default();
    cursor.jump_to_end_of_line(COLS, buf.len());
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
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
#[test]
fn cursor_behind_data_left_nibble_move_1_right() {
    // Create data from 0 to 1 as test data
    let buf: Vec<u8> = (0..2).collect();
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.pos = 2;
    cursor.move_n_right(1, buf.len());
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
    //TODO: Where to be now?
    //assert_eq!(cursor.pos, 2);
}
#[test]
fn cursor_behind_data_right_nibble_move_1_right() {
    // Create data from 0 to 1 as test data
    let buf: Vec<u8> = (0..2).collect();
    // 0x00 0x01
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.pos = 2;
    cursor.move_n_right(1, buf.len());
    assert_eq!(cursor.sel, CursorSelects::RightNibble);
    //TODO: Where to be now?
    //assert_eq!(cursor.pos, 2);
}
#[test]
fn cursor_left_nibble_swap_to_ascii() {
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::LeftNibble;
    cursor.swap_selection_hex_ascii();
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_right_nibble_swap_to_ascii() {
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::RightNibble;
    cursor.swap_selection_hex_ascii();
    assert_eq!(cursor.sel, CursorSelects::AsciiChar);
}
#[test]
fn cursor_ascii_swap_to_hex() {
    let mut cursor = Cursor::default();
    cursor.sel = CursorSelects::AsciiChar;
    cursor.swap_selection_hex_ascii();
    assert_eq!(cursor.sel, CursorSelects::LeftNibble);
}
