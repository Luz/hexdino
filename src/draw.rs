extern crate ncurses;
use ncurses::*;
use std::cmp;
use CursorSelects;
use CursorState;

pub fn draw(
    buf: &[u8],
    cols: usize,
    command: &String,
    debug: &mut String,
    cursor: CursorState,
    screenoffset: usize,
) {
    erase();

    //    debug.clear();
    //    let screensize = get_screen_size(cols);
    //    debug.push_str(&format!("   screensize:{:?}", screensize));
    //    debug.push_str(&format!("   screenoffset:{:?}", screenoffset));
    //    debug.push_str(&format!("   buf.len():{:?}", buf.len()));

    let screenheight: usize;
    screenheight = getmaxy(stdscr()) as usize;
    //    debug.push_str(&format!("   screenheight:{:?}", screenheight));

    let mut tmpbuflen = buf.len();
    if tmpbuflen >= 1 {
        tmpbuflen -= 1;
    }
    let rows = tmpbuflen / cols + 1;

    //    debug.push_str(&format!("   rows:{:?}", rows));

    for z in 0..rows {
        // 8 hex digits (4GB/cols or 0.25GB@cols=COLS)
        addstr(&format!(
            "{:08X}: ",
            get_absolute_line(cols, screenoffset, z)
        ));
        // Additional space between line number and hex
        addstr(" ");
        for s in 0..cols {
            let pos: usize = z * cols + s;
            if pos < buf.len() {
                color_left_nibble_cond(true, pos + cols * screenoffset == cursor.pos, cursor);
                addstr(&format!("{:01X}", buf[pos] >> 4));
                color_left_nibble_cond(false, pos + cols * screenoffset == cursor.pos, cursor);

                color_right_nibble_cond(true, pos + cols * screenoffset == cursor.pos, cursor);
                addstr(&format!("{:01X}", buf[pos] & 0x0F));
                color_right_nibble_cond(false, pos + cols * screenoffset == cursor.pos, cursor);

                addstr(" ");
            } else if pos == buf.len() {
                color_left_nibble_cond(true, pos + cols * screenoffset == cursor.pos, cursor);
                addstr("-");
                color_left_nibble_cond(false, pos + cols * screenoffset == cursor.pos, cursor);

                color_right_nibble_cond(true, pos + cols * screenoffset == cursor.pos, cursor);
                addstr("-");
                color_right_nibble_cond(false, pos + cols * screenoffset == cursor.pos, cursor);

                addstr(" ");
            } else {
                addstr("-- ");
            }
        }
        // Additional space between hex and ascii
        addstr(" ");
        for s in 0..cols {
            let pos: usize = z * cols + s;
            color_ascii_cond(true, pos + cols * screenoffset == cursor.pos, cursor);
            if pos < buf.len() {
                if let c @ 32..=126 = buf[pos] {
                    addstr(&format!("{}", c as char));
                } else {
                    // Mark non-ascii symbols
                    addstr(&format!("."));
                }
            } else if pos == buf.len() {
                // Pad ascii with spaces
                addstr(" ");
            }

            color_ascii_cond(false, pos + cols * screenoffset == cursor.pos, cursor);
        }
        addstr("\n");
    }
    for _ in 1..screenheight - rows {
        // Put the cursor on last line of terminal
        addstr("\n");
    }
    addstr(&format!("{}", command));
    addstr(&format!("{}", debug));
}

fn get_absolute_line(cols: usize, screenoffset: usize, z: usize) -> usize {
    return z * cols + screenoffset * cols;
}
pub fn get_screen_size(cols: usize) -> usize {
    let screenheight: usize;
    screenheight = getmaxy(stdscr()) as usize;

    let mut ret: usize = 0;

    // Last line reserved for Status/Commands/etc (Like in vim)
    if screenheight >= 1 {
        ret = (screenheight - 1) * cols;
    }
    return ret;
}
pub fn get_absolute_draw_indices(
    buflen: usize,
    cols: usize,
    screenoffset: usize,
) -> (usize, usize) {
    let max_draw_len: usize = cmp::min(buflen, get_screen_size(cols)); //hier muss bei get_screen_size noch auf 16er abgerundet werden?

    let starting_pos: usize = screenoffset * cols;
    let mut ending_pos: usize = starting_pos + max_draw_len;
    if ending_pos > buflen {
        ending_pos = buflen;
    }

    return (starting_pos, ending_pos);
}

fn color_left_nibble(color: bool, cursor: CursorState) {
    if color {
        if cursor.sel == CursorSelects::Leftnibble {
            attron(COLOR_PAIR(1) | A_STANDOUT());
        } else if cursor.sel == CursorSelects::Asciichar {
            attron(A_UNDERLINE());
        }
    } else {
        if cursor.sel == CursorSelects::Leftnibble {
            attroff(COLOR_PAIR(1) | A_STANDOUT());
        } else if cursor.sel == CursorSelects::Asciichar {
            attroff(A_UNDERLINE());
        }
    }
}
fn color_left_nibble_cond(color: bool, condition: bool, cursor: CursorState) {
    if condition {
        color_left_nibble(color, cursor);
    }
}

fn color_right_nibble(color: bool, cursor: CursorState) {
    if color {
        if cursor.sel == CursorSelects::Rightnibble {
            attron(COLOR_PAIR(1) | A_STANDOUT());
        } else if cursor.sel == CursorSelects::Asciichar {
            attron(A_UNDERLINE());
        }
    } else {
        if cursor.sel == CursorSelects::Rightnibble {
            attroff(COLOR_PAIR(1) | A_STANDOUT());
        } else if cursor.sel == CursorSelects::Asciichar {
            attroff(A_UNDERLINE());
        }
    }
}
fn color_right_nibble_cond(color: bool, condition: bool, cursor: CursorState) {
    if condition {
        color_right_nibble(color, cursor);
    }
}

fn color_ascii(color: bool, cursor: CursorState) {
    if color {
        if cursor.sel == CursorSelects::Asciichar {
            attron(COLOR_PAIR(1) | A_STANDOUT());
        } else {
            attron(A_UNDERLINE());
        }
    } else {
        if cursor.sel == CursorSelects::Asciichar {
            attroff(COLOR_PAIR(1) | A_STANDOUT());
        } else {
            attroff(A_UNDERLINE());
        }
    }
}
fn color_ascii_cond(color: bool, condition: bool, cursor: CursorState) {
    if condition {
        color_ascii(color, cursor);
    }
}
