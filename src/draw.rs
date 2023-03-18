use anyhow::Error;
use crossterm::{
    cursor, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal,
};
use std::io::prelude::*;
use std::io::stdout;

use super::CursorSelects;
use super::CursorState;

pub fn draw(
    total_buf: &[u8],
    cols: usize,
    command: &String,
    infoline: &mut String,
    cursor: CursorState,
    screenoffset: usize,
) -> Result<(), Error> {
    let draw_range = get_absolute_draw_indices(total_buf.len(), cols, screenoffset);

    let buf = &total_buf[draw_range.0..draw_range.1];
    drop(total_buf);

    let mut out = stdout();
    queue!(out, terminal::Clear(terminal::ClearType::All))?;
    queue!(out, cursor::MoveTo(0, 0))?;

    let screensize = crossterm::terminal::size()?;
    let screenheight: usize = screensize.1 as usize;

    let mut tmpbuflen = buf.len();
    if tmpbuflen >= 1 {
        tmpbuflen -= 1;
    }
    let rows = tmpbuflen / cols + 1;

    for z in 0..rows {
        // 8 hex digits (4GB/cols or 0.25GB@cols=COLS)
        let address: String = format!("{:08X}: ", get_absolute_line(cols, screenoffset, z));
        queue!(out, Print(address))?;
        // Additional space between line number and hex
        queue!(out, Print(" "))?;
        for s in 0..cols {
            let pos: usize = z * cols + s;
            if pos < buf.len() {
                if pos + cols * screenoffset == cursor.pos {
                    color_left_nibble(cursor);
                }
                let left_nibble: String = format!("{:01X}", buf[pos] >> 4);
                queue!(out, Print(left_nibble))?;
                queue!(out, ResetColor)?;

                if pos + cols * screenoffset == cursor.pos {
                    color_right_nibble(cursor);
                }
                let right_nibble: String = format!("{:01X}", buf[pos] & 0x0F);
                queue!(out, Print(right_nibble))?;
                queue!(out, ResetColor)?;

                queue!(out, Print(" "))?;
            } else if pos == buf.len() {
                if pos + cols * screenoffset == cursor.pos {
                    color_left_nibble(cursor);
                }
                queue!(out, Print("-"))?;
                queue!(out, ResetColor)?;

                if pos + cols * screenoffset == cursor.pos {
                    color_right_nibble(cursor);
                }
                queue!(out, Print("-"))?;
                queue!(out, ResetColor)?;

                queue!(out, Print(" "))?;
            } else {
                queue!(out, Print("-- "))?;
            }
        }
        // Additional space between hex and ascii
        queue!(out, Print(" "))?;
        for s in 0..cols {
            let pos: usize = z * cols + s;
            color_ascii(pos + cols * screenoffset == cursor.pos, cursor);
            if pos < buf.len() {
                if let c @ 32..=126 = buf[pos] {
                    let ascii_symbol: String = format!("{}", c as char);
                    queue!(out, Print(ascii_symbol))?;
                } else {
                    // Mark non-ascii symbols
                    queue!(out, Print("."))?;
                }
            } else if pos == buf.len() {
                // Pad ascii with spaces
                queue!(out, Print(" "))?;
            }
            queue!(out, ResetColor)?;
        }
        queue!(out, Print("\n\r"))?;
    }
    for _ in 1..screenheight - rows {
        // Put the cursor on last line of terminal
        queue!(out, Print("\n"))?;
    }
    queue!(out, Print(command))?;
    queue!(out, Print(infoline))?;
    out.flush()?;
    Ok(())
}

fn get_absolute_line(cols: usize, screenoffset: usize, z: usize) -> usize {
    return z * cols + screenoffset * cols;
}
fn get_screen_size(cols: usize) -> usize {
    let screensize = crossterm::terminal::size().unwrap_or_default();
    let screenheight: usize = screensize.1 as usize;

    let mut ret: usize = 0;

    // Last line reserved for Status/Commands/etc (Like in vim)
    if screenheight >= 1 {
        ret = (screenheight - 1) * cols;
    }
    return ret;
}
fn get_absolute_draw_indices(buflen: usize, cols: usize, screenoffset: usize) -> (usize, usize) {
    // Do we need to round() down to 16 when using get_screen_size()?
    let max_draw_len: usize = std::cmp::min(buflen, get_screen_size(cols));

    let starting_pos: usize = screenoffset * cols;
    let mut ending_pos: usize = starting_pos + max_draw_len;
    if ending_pos > buflen {
        ending_pos = buflen;
    }

    return (starting_pos, ending_pos);
}

fn color_left_nibble(cursor: CursorState) {
    if cursor.sel == CursorSelects::LeftNibble {
        color_cursor();
    } else if cursor.sel == CursorSelects::AsciiChar {
        underline();
    }
}

fn color_right_nibble(cursor: CursorState) {
    if cursor.sel == CursorSelects::RightNibble {
        color_cursor();
    } else if cursor.sel == CursorSelects::AsciiChar {
        underline();
    }
}

fn color_ascii(condition: bool, cursor: CursorState) {
    if condition {
        if cursor.sel == CursorSelects::AsciiChar {
            color_cursor();
        } else {
            underline();
        }
    }
}
// This is the actual cursor
fn color_cursor() {
    queue!(
        stdout(),
        SetBackgroundColor(Color::Green),
        SetForegroundColor(Color::Black)
    )
    .unwrap_or(());
}
fn underline() {
    queue!(stdout(), SetAttribute(Attribute::Underlined)).unwrap_or(());
}
