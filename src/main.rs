//! # Hexdino
//!
//! A hex editor with vim like keybindings written in Rust.

#![doc(html_logo_url = "https://raw.githubusercontent.com/Luz/hexdino/master/logo.png")]
#![deny(trivial_casts)]
#![allow(unused_variable)]

use std::cmp;
use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::stdout;
use std::io::SeekFrom;
use std::path::Path;

mod draw;
use draw::draw;
use draw::get_absolute_draw_indices;
mod find;
use find::FindOptSubset;

extern crate crossterm;
use crossterm::event::{read, Event};
use crossterm::{
    cursor, queue,
    style::Print,
    terminal,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

mod keycodes;
use keycodes::extract;

extern crate getopts;
use getopts::Options;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "cmd.pest"]
struct CmdParser;

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

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILENAME [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() -> Result<()> {
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
    let mut autoparse = String::new();
    let mut infoline = String::new();

    std::thread::sleep(std::time::Duration::from_secs(5));

    let mut out = stdout();
    let screensize = crossterm::terminal::size()?;
    let screenheight: usize = screensize.1 as usize;

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "version", "print the version");
    let arg_matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            print_usage(&program, opts);
            std::process::exit(-1);
        }
    };
    if arg_matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(0);
    }
    if arg_matches.opt_present("v") {
        println!("Name: {}", env!("CARGO_PKG_NAME"));
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
        std::process::exit(0);
    }

    let arg_filename = match arg_matches.free.is_empty() {
        true => String::new(),
        false => arg_matches.free[0].clone(),
    };
    if arg_filename.is_empty() {
        println!("FILENAME is empty!\n");
        print_usage(&program, opts);
        std::process::exit(-1);
    }
    let path = Path::new(&arg_filename);

    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
    {
        Err(why) => {
            println!("Could not open {}: {}", path.display(), why.to_string());
            std::process::exit(-1);
        }
        Ok(file) => file,
    };
    file.read_to_end(&mut buf)
        .ok()
        .expect("File could not be read.");

    queue!(out, terminal::Clear(terminal::ClearType::All))?;
    queue!(out, cursor::MoveTo(0, 0))?;
    queue!(out, Print("Screenheight is ".to_string()))?;
    queue!(out, Print(screenheight.to_string()))?;
    queue!(out, Print("\n\r"))?;
    out.flush()?;

    enable_raw_mode()?;

    let draw_range = get_absolute_draw_indices(buf.len(), COLS, screenoffset);
    draw(
        &buf[draw_range.0..draw_range.1],
        COLS,
        &command,
        &mut infoline,
        cursor,
        screenoffset,
    )?;

    let mut quitnow = false;
    while quitnow == false {
        if autoparse.is_empty() {
            let key = read()?;
            let mut keycode: char = '\u{00}';
            // This is close to the old c-style 'getch()':
            match key {
                Event::Key(event) => {
                    keycode = extract(event.code).unwrap_or('\u{00}');
                }
                Event::Mouse(_event) => (), // This can be handled later
                Event::FocusGained => (),   // This can be handled later
                Event::FocusLost => (),     // This can be handled later
                Event::Paste(_text) => (),  // This can be handled later
                Event::Resize(_width, _height) => (), // This can be handled later
            };
            command.push_str(&keycode.clone().to_string());
        } else {
            command.push(autoparse.chars().nth(0).unwrap());
            autoparse.remove(0);
        }

        let parsethisstring = command.clone();
        let cmd = CmdParser::parse(Rule::cmd_list, &parsethisstring)
            .unwrap()
            .next()
            .unwrap();

        let mut clear = true;
        let mut save = false;

        match cmd.as_rule() {
            Rule::down => {
                if cursor.pos + COLS < buf.len() {
                    // not at end
                    cursor.pos += COLS;
                } else {
                    // when at end
                    cursor.pos = buf.len().saturating_sub(1);
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
                    cursor.pos = buf.len().saturating_sub(1);
                }
                if cursor.sel == CursorSelects::LeftNibble {
                    cursor.sel = CursorSelects::RightNibble;
                }
            }
            Rule::bottom => {
                cursor.pos = buf.len().saturating_sub(1);
                cursor.pos -= cursor.pos % COLS; // jump to start of line
            }
            Rule::replace => {
                clear = false;
            }
            Rule::replacement => {
                let key = command.chars().last().unwrap_or('x');
                if cursor.sel == CursorSelects::AsciiChar {
                    if cursor.pos >= buf.len() {
                        buf.insert(cursor.pos, 0);
                    }
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
                lastcommand = command.clone();
                clear = true;
            }
            Rule::replaceend => {
                clear = true;
            }
            Rule::remove => {
                // check if in valid range
                if buf.len() > 0 && cursor.pos < buf.len() {
                    // remove the current char
                    buf.remove(cursor.pos);
                }
                // Move left if cursor is currently out of data
                if cursor.pos >= buf.len() {
                    cursor.pos = cursor.pos.saturating_sub(1);
                }
                lastcommand = command.clone();
            }
            Rule::dd => {
                let amount: usize = cmd.as_str().parse().unwrap_or(1);
                if cursor.pos < buf.len() {
                    let startofline = cursor.pos - cursor.pos % COLS;
                    let mut endofline = cursor.pos - (cursor.pos % COLS) + (COLS * amount);
                    endofline = cmp::min(endofline, buf.len());
                    buf.drain(startofline..endofline);
                    if cursor.pos >= buf.len() {
                        cursor.pos = buf.len().saturating_sub(1);
                    }
                }
                lastcommand = command.clone();
                clear = true;
            }
            Rule::insert => {
                // The next chars will be inserted
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
                lastcommand = command.clone();
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
                autoparse = lastcommand.clone();
            }
            Rule::gg => {
                let linenr: usize = cmd.as_str().parse().unwrap_or(0);
                cursor.pos = linenr * COLS; // jump to the line
                if cursor.pos > buf.len() {
                    // detect file end
                    cursor.pos = buf.len();
                }
                cursor.pos -= cursor.pos % COLS; // jump to start of line
                clear = true;
            }
            Rule::searchend => {
                let searchstr = cmd.clone().into_inner().as_str();
                let search = searchstr.as_bytes();
                let foundpos = TwoWaySearcher::new(&search);
                cursor.pos = foundpos.search_in(&buf).unwrap_or(cursor.pos);
                clear = true;
            }
            Rule::hexsearchend => {
                let searchbytes = cmd.clone().into_inner().as_str();
                let search = searchbytes.as_bytes();
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
                // infoline.push_str(&format!("Searching for: {:?}", needle ));
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
            Rule::escape => (),
            Rule::gatherall => {
                // When the command is still to be fully built
                clear = false;
            }
            _ => (),
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

        let draw_range = get_absolute_draw_indices(buf.len(), COLS, screenoffset);
        draw(
            &buf[draw_range.0..draw_range.1],
            COLS,
            &command,
            &mut infoline,
            cursor,
            screenoffset,
        )?;
    }

    disable_raw_mode()?;
    Ok(())
}
