extern crate ncurses;
use ncurses::*;

pub fn draw(buf:&Vec<u8>, cursorpos:usize, cols:usize, mode:usize, command:&String, cursorstate:usize, screenoffset:usize) {
    erase();

    let screenheight : usize = getmaxy(stdscr) as usize;

    let mut rows = buf.len() / cols;
    if rows >= screenheight-1 { // Last line reserved for Status/Commands/etc (Like in vim)
        rows = screenheight-2;
    }

    for z in 0 .. rows+1 {
        printw(&format!("{:08X}: ", get_line(cols, screenoffset, z))); // 8 hex digits (4GB/cols or 0.25GB@cols=SPALTEN)
        printw(" "); // Additional space between line number and hex
        for s in 0 .. cols {
            let pos:usize = get_pos(cols, screenoffset, z, s);
            if pos < buf.len() {

                color_left_nibble_cond(true, pos==cursorpos, cursorstate);
                printw(&format!("{:01X}", buf[pos]>>4) );
                color_left_nibble_cond(false, pos==cursorpos, cursorstate);

                color_right_nibble_cond(true, pos==cursorpos, cursorstate);
                printw(&format!("{:01X}", buf[pos]&0x0F) );
                color_right_nibble_cond(false, pos==cursorpos, cursorstate);

                printw(" ");
            } else
            if pos == buf.len() {

                color_left_nibble_cond(true, pos==cursorpos, cursorstate);
                printw("-");
                color_left_nibble_cond(false, pos==cursorpos, cursorstate);

                color_right_nibble_cond(true, pos==cursorpos, cursorstate);
                printw("-");
                color_right_nibble_cond(false, pos==cursorpos, cursorstate);

                printw(" ");
            } else {
                printw("-- ");
            }
        }
        printw(" "); // Additional space between hex and ascii
        for s in 0 .. cols {
            let pos:usize = get_pos(cols, screenoffset, z, s);
            color_ascii_cond(true, pos==cursorpos, cursorstate);
            if pos < buf.len() {
                if let c @ 32...126 = buf[pos] {
                    if c as char == '%' {
                        printw("%%"); // '%' needs to be escaped by a '%' in ncurses
                    } else {
                        printw(&format!("{}", c as char) );
                    }
                }
                else {printw(&format!(".") );} // Mark non-ascii symbols
            } else
            if pos == buf.len() {
                printw(" "); // Pad ascii with spaces
            }

            color_ascii_cond(false, pos==cursorpos, cursorstate);
        }
        printw("\n");
    }
    for _ in 0 .. screenheight-rows-2 { // TODO: check if "rows" is better
        printw("\n"); // Put the cursor on last line of terminal
    }
    if mode == 2 {
        printw(":"); // Indicate that a command can be typed in
    }
    if mode == 3 {
        printw("insert"); // Indicate that insert mode is active
    }
    if mode == 4 {
        printw("/"); // indicate that the search mode is active
    }
    printw(&format!("{}", command));
}

fn get_line(cols:usize, screenoffset:usize, z:usize) -> usize {
    return z*cols+screenoffset*cols;
}
fn get_pos(cols:usize, screenoffset:usize, z:usize, s:usize) -> usize {
    return z*cols+screenoffset*cols+s;
}

fn color_left_nibble(color:bool, cursorstate:usize) {
    if color {
        if cursorstate == 0 {attron(COLOR_PAIR(1) | A_STANDOUT());}
        else if cursorstate == 2 {attron(A_UNDERLINE());}
    } else {
        if cursorstate == 0 {attroff(COLOR_PAIR(1) | A_STANDOUT());}
        else if cursorstate == 2 {attroff(A_UNDERLINE());}
    }
}
fn color_left_nibble_cond(color:bool, condition:bool, cursorstate:usize) {
    if condition { color_left_nibble(color, cursorstate); }
}

fn color_right_nibble(color:bool, cursorstate:usize) {
    if color {
        if cursorstate == 1 {attron(COLOR_PAIR(1) | A_STANDOUT());}
        else if cursorstate == 2 {attron(A_UNDERLINE());}
    } else {
        if cursorstate == 1 {attroff(COLOR_PAIR(1) | A_STANDOUT());}
        else if cursorstate == 2 {attroff(A_UNDERLINE());}
    }
}
fn color_right_nibble_cond(color:bool, condition:bool, cursorstate:usize) {
    if condition { color_right_nibble(color, cursorstate); }
}

fn color_ascii(color:bool, cursorstate:usize) {
    if color {
        if cursorstate == 2 {attron(COLOR_PAIR(1) | A_STANDOUT());}
        else {attron(A_UNDERLINE());}
    } else {
        if cursorstate == 2 {attroff(COLOR_PAIR(1) | A_STANDOUT());}
        else {attroff(A_UNDERLINE());}
    }
}
fn color_ascii_cond(color:bool, condition:bool, cursorstate:usize) {
    if condition { color_ascii(color, cursorstate); }
}
