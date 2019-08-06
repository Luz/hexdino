extern crate ncurses;
use ncurses::*;
use Cursorstate;
use std::cmp;

pub fn draw(
    buf: &Vec<u8>,
    cursorpos: usize,
    cols: usize,
    command: &String,
    debug: &mut String,
    cstate: Cursorstate,
    screenoffset: usize,
) {
    erase();


    // TODO: just temporary, use this later to pick out "&Vec<u8>" from draw(..)-function:
    debug.clear();
    let draw_range = get_draw_indices(buf.len(), cols, screenoffset);
    debug.push_str(&format!("          draw_range:{:?}", draw_range));
    let screensize = get_screen_size(cols);
    debug.push_str(&format!("   screensize:{:?}", screensize));


    let screenheight: usize;
    screenheight = getmaxy(stdscr()) as usize;

    let mut rows = buf.len() / cols;
    // Last line reserved for Status/Commands/etc (Like in vim)
    if rows >= screenheight - 1 {
        rows = screenheight - 2; //TODO: this shall not be less than 0 or it panicks?
    }

    for z in 0..rows + 1 {
        // 8 hex digits (4GB/cols or 0.25GB@cols=SPALTEN)
        printw(&format!("{:08X}: ", get_line(cols, screenoffset, z)));
        // Additional space between line number and hex
        printw(" ");
        for s in 0..cols {
            let pos: usize = get_pos(cols, screenoffset, z, s);
            if pos < buf.len() {

                color_left_nibble_cond(true, pos == cursorpos, cstate);
                printw(&format!("{:01X}", buf[pos] >> 4));
                color_left_nibble_cond(false, pos == cursorpos, cstate);

                color_right_nibble_cond(true, pos == cursorpos, cstate);
                printw(&format!("{:01X}", buf[pos] & 0x0F));
                color_right_nibble_cond(false, pos == cursorpos, cstate);

                printw(" ");
            } else if pos == buf.len() {

                color_left_nibble_cond(true, pos == cursorpos, cstate);
                printw("-");
                color_left_nibble_cond(false, pos == cursorpos, cstate);

                color_right_nibble_cond(true, pos == cursorpos, cstate);
                printw("-");
                color_right_nibble_cond(false, pos == cursorpos, cstate);

                printw(" ");
            } else {
                printw("-- ");
            }
        }
        // Additional space between hex and ascii
        printw(" ");
        for s in 0..cols {
            let pos: usize = get_pos(cols, screenoffset, z, s);
            color_ascii_cond(true, pos == cursorpos, cstate);
            if pos < buf.len() {
                if let c @ 32...126 = buf[pos] {
                    if c as char == '%' {
                        // '%' needs to be escaped by a '%' in ncurses
                        printw("%%");
                    } else {
                        printw(&format!("{}", c as char));
                    }
                } else {
                    // Mark non-ascii symbols
                    printw(&format!("."));
                }
            } else if pos == buf.len() {
                // Pad ascii with spaces
                printw(" ");
            }

            color_ascii_cond(false, pos == cursorpos, cstate);
        }
        printw("\n");
    }
    // TODO: check if "rows" is better
    for _ in 0..screenheight - rows - 2 {
        // Put the cursor on last line of terminal
        printw("\n");
    }
    printw(&format!("{}", command));
    printw(&format!("{}", debug));
}

fn get_line(cols: usize, screenoffset: usize, z: usize) -> usize {
    return z * cols + screenoffset * cols;
}
fn get_pos(cols: usize, screenoffset: usize, z: usize, s: usize) -> usize {
    return z * cols + screenoffset * cols + s;
}
pub fn get_screen_size(
    cols: usize,
    ) -> usize {
    let screenheight: usize;
    screenheight = getmaxy(stdscr()) as usize;

    let mut ret: usize = 0;

    // Last line reserved for Status/Commands/etc (Like in vim)
    if screenheight >= 1 {
        ret = (screenheight-1) * cols;
    }
    return ret;
}
pub fn get_draw_indices(
    buflen: usize,
    cols: usize,
    screenoffset: usize,
    ) -> (usize, usize) {

    let max_draw_len:usize = cmp::min(buflen, get_screen_size(cols));

    let starting_pos: usize = screenoffset * cols;
    let ending_pos: usize = starting_pos + max_draw_len;

    return (starting_pos, ending_pos);
}

fn color_left_nibble(color: bool, cstate: Cursorstate) {
    if color {
        if cstate == Cursorstate::Leftnibble {
            attron(COLOR_PAIR(1) | A_STANDOUT());
        } else if cstate == Cursorstate::Asciichar {
            attron(A_UNDERLINE());
        }
    } else {
        if cstate == Cursorstate::Leftnibble {
            attroff(COLOR_PAIR(1) | A_STANDOUT());
        } else if cstate == Cursorstate::Asciichar {
            attroff(A_UNDERLINE());
        }
    }
}
fn color_left_nibble_cond(color: bool, condition: bool, cstate: Cursorstate) {
    if condition {
        color_left_nibble(color, cstate);
    }
}

fn color_right_nibble(color: bool, cstate: Cursorstate) {
    if color {
        if cstate == Cursorstate::Rightnibble {
            attron(COLOR_PAIR(1) | A_STANDOUT());
        } else if cstate == Cursorstate::Asciichar {
            attron(A_UNDERLINE());
        }
    } else {
        if cstate == Cursorstate::Rightnibble {
            attroff(COLOR_PAIR(1) | A_STANDOUT());
        } else if cstate == Cursorstate::Asciichar {
            attroff(A_UNDERLINE());
        }
    }
}
fn color_right_nibble_cond(color: bool, condition: bool, cstate: Cursorstate) {
    if condition {
        color_right_nibble(color, cstate);
    }
}

fn color_ascii(color: bool, cstate: Cursorstate) {
    if color {
        if cstate == Cursorstate::Asciichar {
            attron(COLOR_PAIR(1) | A_STANDOUT());
        } else {
            attron(A_UNDERLINE());
        }
    } else {
        if cstate == Cursorstate::Asciichar {
            attroff(COLOR_PAIR(1) | A_STANDOUT());
        } else {
            attroff(A_UNDERLINE());
        }
    }
}
fn color_ascii_cond(color: bool, condition: bool, cstate: Cursorstate) {
    if condition {
        color_ascii(color, cstate);
    }
}
