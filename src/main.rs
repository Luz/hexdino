//! # Hexdino
//!
//! A hex editor with vim like keybindings written in Rust.

#![doc(html_logo_url = "https://raw.githubusercontent.com/Luz/hexdino/master/logo.png")]
#![deny(trivial_casts)]

use std::cmp;
use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;

mod draw;
use draw::draw;
use draw::get_absolute_draw_indices;
mod find;
use find::FindOptSubset;

extern crate ncurses;
use ncurses::*;

extern crate getopts;
use getopts::Options;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "cmd.pest"]
struct IdentParser;

extern crate memmem;
use memmem::{Searcher, TwoWaySearcher};

#[derive(PartialEq, Copy, Clone)]
pub enum CursorSelects {
    LeftNibble,
    RightNibble,
    AsciiChar,
}

#[derive(Copy, Clone)]
pub struct CursorState {
    pos: usize,
    sel: CursorSelects,
}

fn main() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let mut buf = vec![];
    let mut cursor = CursorState {
        pos: 0,
        sel: CursorSelects::LeftNibble,
    };
    // 0 = display data from first line of file
    let mut screenoffset: usize = 0;
    const COLS: usize = 16;
    let mut command = String::new();
    let mut lastcommand = String::new();
    let mut debug = String::new();

    // start ncursesw
    initscr();
    let screenheight = getmaxy(stdscr()) as usize;
    // ctrl+z and fg works with this
    cbreak();
    noecho();
    start_color();
    use_default_colors();
    init_pair(1, COLOR_GREEN, -1);

    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "version", "print the version");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            endwin();
            println!("{}", f.to_string());
            println!("Usage: {} FILE [options]", program);
            return;
        }
    };
    if matches.opt_present("v") {
        endwin();
        println!("Version: {}", VERSION);
        return;
    }
    if matches.opt_present("h") {
        endwin();
        println!("Usage: {} FILE [options]", program);
        return;
    }

    if !has_colors() {
        endwin();
        println!("Your terminal does not support color!\n");
        return;
    }

    let patharg = match matches.free.is_empty() {
        true => String::new(),
        false => matches.free[0].clone(),
    };
    let path = Path::new(&patharg);

    if patharg.is_empty() {
        endwin();
        println!("Patharg is empty!\n");
        return;
    }

    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
    {
        Err(why) => {
            endwin();
            println!("Could not open {}: {}", path.display(), why.to_string());
            return;
        }
        Ok(file) => file,
    };
    file.read_to_end(&mut buf)
        .ok()
        .expect("File could not be read.");

    let draw_range = get_absolute_draw_indices(buf.len(), COLS, screenoffset);
    draw(
        &buf[draw_range.0..draw_range.1],
        COLS,
        &command,
        &mut debug,
        cursor,
        screenoffset,
    );

    let mut quitnow = false;
    let mut autoparse = false;
    while quitnow == false {
        if !autoparse {
            let key = std::char::from_u32(getch() as u32).unwrap();
            command.push_str(&key.clone().to_string());
        }
        autoparse = false;

        let parsethisstring = command.clone();
        let commands = IdentParser::parse(Rule::cmd_list, &parsethisstring)
            .unwrap_or_else(|e| panic!("{}", e));

        let mut clear = true;
        let mut save = false;
        for cmd in commands {
            match cmd.as_rule() {
                Rule::down => {
                    // debug.push_str(&format!("{:?}", cmd.as_rule()));
                    if cursor.pos + COLS < buf.len() {
                        // not at end
                        cursor.pos += COLS;
                    } else {
                        // when at end
                        if buf.len() != 0 {
                            // Suppress underflow
                            cursor.pos = buf.len() - 1;
                        }
                    }
                }
                Rule::up => {
                    if cursor.pos >= COLS {
                        cursor.pos -= COLS;
                    }
                }
                Rule::left => {
                    if cursor.sel == CursorSelects::AsciiChar {
                        if cursor.pos > 0 {
                            cursor.pos -= 1;
                        }
                    } else if cursor.sel == CursorSelects::RightNibble {
                        cursor.sel = CursorSelects::LeftNibble;
                    } else if cursor.sel == CursorSelects::LeftNibble {
                        if cursor.pos > 0 {
                            // not at start
                            cursor.sel = CursorSelects::RightNibble;
                            cursor.pos -= 1;
                        }
                    }
                }
                Rule::right => {
                    if cursor.sel == CursorSelects::AsciiChar {
                        if cursor.pos + 1 < buf.len() {
                            // not at end
                            cursor.pos += 1;
                        }
                    } else if cursor.sel == CursorSelects::LeftNibble {
                        cursor.sel = CursorSelects::RightNibble;
                    } else if cursor.sel == CursorSelects::RightNibble {
                        if cursor.pos + 1 < buf.len() {
                            // not at end
                            cursor.sel = CursorSelects::LeftNibble;
                            cursor.pos += 1;
                        }
                    }
                }
                Rule::start => {
                    cursor.pos -= cursor.pos % COLS; // jump to start of line
                    if cursor.sel == CursorSelects::RightNibble {
                        cursor.sel = CursorSelects::LeftNibble;
                    }
                }
                Rule::end => {
                    // check if no overflow
                    if cursor.pos - (cursor.pos % COLS) + (COLS - 1) < buf.len() {
                        // jump to end of line
                        cursor.pos = cursor.pos - (cursor.pos % COLS) + (COLS - 1);
                    } else {
                        // jump to end of line
                        cursor.pos = buf.len() - 1
                    }
                    if cursor.sel == CursorSelects::LeftNibble {
                        cursor.sel = CursorSelects::RightNibble;
                    }
                }
                Rule::bottom => {
                    cursor.pos = buf.len() - 1;
                    cursor.pos -= cursor.pos % COLS; // jump to start of line
                }
                Rule::replace => {
                    // debug.push_str("next char will be the replacement!");
                    clear = false;
                }
                Rule::remove => {
                    // check if in valid range
                    if buf.len() > 0 && cursor.pos < buf.len() {
                        // remove the current char
                        buf.remove(cursor.pos);
                    }
                    // always perform the movement if possible
                    if cursor.pos > 0 && cursor.pos >= buf.len() {
                        cursor.pos -= 1;
                    }
                    lastcommand = command.clone();
                }
                Rule::insert => {
                    // debug.push_str("next chars will be inserted!");
                    clear = false;
                }
                Rule::insertstuff => {
                    let key = command.chars().last().unwrap_or('x');

                    if cursor.sel == CursorSelects::LeftNibble {
                        // Left nibble
                        if let Some(c) = key.to_digit(16) {
                            buf.insert(cursor.pos, (c as u8) << 4);
                            cursor.sel = CursorSelects::RightNibble;
                        }
                    } else if cursor.sel == CursorSelects::RightNibble {
                        // Right nibble
                        if cursor.pos == buf.len() {
                            buf.insert(cursor.pos, 0);
                        }
                        if let Some(c) = key.to_digit(16) {
                            buf[cursor.pos] = buf[cursor.pos] & 0xF0 | c as u8;
                            cursor.sel = CursorSelects::LeftNibble;
                            cursor.pos += 1;
                        }
                    } else if cursor.sel == CursorSelects::AsciiChar {
                        buf.insert(cursor.pos, key as u8);
                        cursor.pos += 1;
                    }

                    clear = false;
                }
                Rule::insertend => {
                    // debug.push_str(&format!("Insert ended. ({:?})", command.clone()));
                    lastcommand.clear();
                    lastcommand.push_str(&format!(
                        "Command repeation for {:?} not yet implemented.",
                        cmd.as_rule()
                    ));
                    clear = true;
                }
                Rule::jumpascii => {
                    if cursor.sel == CursorSelects::AsciiChar {
                        cursor.sel = CursorSelects::LeftNibble;
                    } else {
                        cursor.sel = CursorSelects::AsciiChar;
                    }
                }
                Rule::helpfile => {
                    command.pop();
                    command.push_str("No helpfile yet");
                    clear = false;
                }
                Rule::repeat => {
                    command = lastcommand.clone();
                    clear = false;
                    autoparse = true;
                }
                Rule::backspace => {
                    command.pop();
                    command.pop();
                    clear = false;
                }
                Rule::saveandexit => {
                    save = true;
                    quitnow = true;
                }
                Rule::exit => quitnow = true,
                Rule::save => save = true,

                _ => (),
            }

            for inner_cmd in cmd.into_inner() {
                match inner_cmd.as_rule() {
                    Rule::replacement => {
                        let key = inner_cmd.as_str().chars().nth(0).unwrap_or('x');
                        if cursor.sel == CursorSelects::AsciiChar {
                            if cursor.pos >= buf.len() {
                                buf.insert(cursor.pos, 0);
                            }
                            // buf[cursor.pos] = inner_cmd.as_str();
                            buf[cursor.pos] = key as u8;
                        } else {
                            let mask = if cursor.sel == CursorSelects::LeftNibble {
                                0x0F
                            } else {
                                0xF0
                            };
                            let shift = if cursor.sel == CursorSelects::LeftNibble {
                                4
                            } else {
                                0
                            };
                            if cursor.pos >= buf.len() {
                                buf.insert(cursor.pos, 0);
                            }
                            // Change the selected nibble
                            if let Some(c) = key.to_digit(16) {
                                buf[cursor.pos] = buf[cursor.pos] & mask | (c as u8) << shift;
                            }
                        }
                        lastcommand.clear();
                        lastcommand.push_str(&format!(
                            "{}{}",
                            command.chars().nth(0).unwrap_or('?'),
                            key
                        ));
                    }
                    Rule::dd_lines => {
                        let amount: usize = inner_cmd.as_str().parse().unwrap_or(1);
                        // check if in valid range
                        if buf.len() > 0 && cursor.pos < buf.len() {
                            let startofline = cursor.pos - cursor.pos % COLS;
                            let mut endofline = cursor.pos - (cursor.pos % COLS) + (COLS * amount);
                            endofline = cmp::min(endofline, buf.len());
                            buf.drain(startofline..endofline);
                            if buf.len() > 0 && cursor.pos >= buf.len() {
                                cursor.pos = buf.len() - 1;
                            }
                        }
                        lastcommand = command.clone();
                    }
                    Rule::searchstr => {
                        let search = inner_cmd.as_str().as_bytes();
                        let foundpos = TwoWaySearcher::new(&search);
                        cursor.pos = foundpos.search_in(&buf).unwrap_or(cursor.pos);
                    }
                    Rule::searchbytes => {
                        let search = inner_cmd.as_str().as_bytes();
                        let mut needle = vec![];
                        for i in 0..search.len() {
                            let nibble = match search[i] as u8 {
                                c @ 48..=57 => c - 48, // Numbers from 0 to 9
                                b'x' => 0x10,          // x is the wildcard
                                b'X' => 0x10,          // X is the wildcard
                                c @ b'a'..=b'f' => c - 87,
                                c @ b'A'..=b'F' => c - 55,
                                _ => panic!("Should not get to this position!"),
                            };
                            needle.push(nibble);
                        }
                        cursor.pos = buf.find_subset(&needle).unwrap_or(cursor.pos);
                        // debug.push_str(&format!("Searching for: {:?}", needle ));
                    }
                    Rule::gg_line => {
                        let linenr: usize = inner_cmd.as_str().parse().unwrap_or(0);
                        cursor.pos = linenr * COLS; // jump to the line
                        if cursor.pos > buf.len() {
                            // detect file end
                            cursor.pos = buf.len();
                        }
                        cursor.pos -= cursor.pos % COLS; // jump to start of line
                    }
                    Rule::escape => (),
                    Rule::gatherone => clear = false,
                    _ => {
                        command.push_str(&format!("no rule for {:?} ", inner_cmd.as_rule()));
                        clear = false;
                    }
                };
            }
            if save {
                if path.exists() {
                    let mut file = match OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(&path)
                    {
                        Err(why) => {
                            panic!("Could not open {}: {}", path.display(), why.to_string())
                        }
                        Ok(file) => file,
                    };
                    file.seek(SeekFrom::Start(0))
                        .ok()
                        .expect("Filepointer could not be set to 0");
                    file.write_all(&mut buf)
                        .ok()
                        .expect("File could not be written.");
                    file.set_len(buf.len() as u64)
                        .ok()
                        .expect("File could not be set to correct lenght.");
                    command.push_str("File saved!");
                } else {
                    command.push_str("Careful, file could not be saved!");
                }
                // TODO: define filename during runtime
                save = false;
            }
            if clear {
                command.clear();
            }

            // Always move screen when cursor leaves screen
            if cursor.pos > (screenheight + screenoffset - 1) * COLS - 1 {
                screenoffset = 2 + cursor.pos / COLS - screenheight;
            }
            if cursor.pos < screenoffset * COLS {
                screenoffset = cursor.pos / COLS;
            }
        }

        let draw_range = get_absolute_draw_indices(buf.len(), COLS, screenoffset);
        draw(
            &buf[draw_range.0..draw_range.1],
            COLS,
            &command,
            &mut debug,
            cursor,
            screenoffset,
        );
    }

    refresh();
    endwin();
}
