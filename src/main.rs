//! # Hexdino
//!
//! A hex editor with vim like keybindings written in Rust.
#![doc(html_logo_url = "https://raw.githubusercontent.com/Luz/hexdino/master/logo.png")]

use anyhow::{Context, Error};
use clap::Parser as ArgParser;
use std::cmp;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

mod draw;
use draw::draw;

mod search;
use search::*;

use memmem::{Searcher, TwoWaySearcher};

use crossterm::event::{read, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod keycodes;
use pest::Parser;
use pest_derive::*;
#[derive(Parser)]
#[grammar = "cmd.pest"]
struct CmdParser;

mod cursor;
use cursor::*;

#[derive(ArgParser)]
#[clap(version, long_about = None)]
struct Args {
    // This is always required for now, as we have no commands to load a file
    #[clap(required = true, value_parser)]
    filename: PathBuf,
    /// Load commands via argument (E.g.: --autoparse=$'jjxx:q\r' )
    #[clap(short, long, default_value = "")]
    autoparse: String,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let mut buf = Vec::new();
    let mut cursor = Cursor::default();
    // 0 = display data from first line of file
    let mut screenoffset: usize = 0;
    const COLS: usize = 16;
    let mut command = String::new();
    let mut lastcommand = String::new();
    let mut autoparse = args.autoparse;
    let mut infotext = String::new();

    let screensize = crossterm::terminal::size()?;
    let screenheight: usize = screensize.1 as usize;

    let path = Path::new(&args.filename);

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .context("File could not be opened.")?;

    file.read_to_end(&mut buf)?;

    enable_raw_mode()?;
    draw(&buf, COLS, &command, &infotext, cursor, screenoffset)?;

    let mut quitnow = false;
    while !quitnow {
        if autoparse.is_empty() {
            let key = read()?;
            let mut keycode: char = '\u{00}';
            // This is close to the old c-style 'getch()':
            match key {
                Event::Key(event) => {
                    keycode = keycodes::extract(event.code).unwrap_or('\u{00}');
                }
                Event::Mouse(_event) => (), // This can be handled later
                Event::FocusGained => (),   // This can be handled later
                Event::FocusLost => (),     // This can be handled later
                Event::Paste(_text) => (),  // This can be handled later
                Event::Resize(_width, _height) => (), // This can be handled later
            };
            command.push_str(&keycode.clone().to_string());
        } else {
            command.push(autoparse.chars().next().unwrap());
            autoparse.remove(0);
        }

        let parsethisstring = command.clone();
        let cmd = CmdParser::parse(Rule::cmd_list, &parsethisstring)
            .unwrap()
            .next()
            .unwrap();

        let mut clear = true;
        let mut save = false;

        // Info can always be cleared as soon as there is a new input
        infotext.clear();

        match cmd.as_rule() {
            Rule::down => {
                let amount: usize = cmd.as_str().parse().unwrap_or(1);
                cursor.move_n_down(amount, COLS, buf.len());
            }
            Rule::up => {
                let amount: usize = cmd.as_str().parse().unwrap_or(1);
                cursor.move_n_up(amount, COLS, buf.len());
            }
            Rule::left => {
                let amount: usize = cmd.as_str().parse().unwrap_or(1);
                cursor.move_n_left(amount);
            }
            Rule::right => {
                let amount: usize = cmd.as_str().parse().unwrap_or(1);
                cursor.move_n_right(amount, buf.len());
            }
            Rule::bottom => {
                let lastline = cursor.get_last_line(COLS, buf.len());
                let line: usize = cmd.as_str().parse().unwrap_or(lastline);
                cursor.move_to_line(line, COLS, buf.len());
            }
            Rule::top => {
                let line: usize = cmd.as_str().parse().unwrap_or(0);
                cursor.move_to_line(line, COLS, buf.len());
            }
            Rule::start => {
                cursor.jump_to_start_of_line(COLS);
            }
            Rule::end => {
                cursor.jump_to_end_of_line(COLS, buf.len());
            }
            Rule::replace => {
                clear = false;
            }
            Rule::replacement => {
                let key = command.chars().last().unwrap_or('x');

                // Allow inserting stuff behind end of buffer
                if cursor.pos() >= buf.len() {
                    buf.insert(cursor.pos(), 0);
                }
                // Insert the key at the selected position
                match cursor.selects() {
                    CursorSelects::AsciiChar => {
                        buf[cursor.pos()] = key as u8;
                    }
                    CursorSelects::LeftNibble => {
                        if let Some(c) = key.to_digit(16) {
                            buf[cursor.pos()] = buf[cursor.pos()] & 0x0F | (c as u8) << 4;
                        }
                    }
                    CursorSelects::RightNibble => {
                        if let Some(c) = key.to_digit(16) {
                            buf[cursor.pos()] = buf[cursor.pos()] & 0xF0 | (c as u8);
                        }
                    }
                }
                lastcommand = command.clone();
            }
            Rule::remove => {
                let amount: usize = cmd.as_str().parse().unwrap_or(1);
                let mut start = cursor.pos();
                let mut end = start + amount;
                start = cmp::min(start, buf.len());
                end = cmp::min(end, buf.len());
                buf.drain(start..end);
                // Move cursor if it is out of data
                cursor.trim_to_max_minus_one(buf.len());
                lastcommand = command.clone();
            }
            Rule::dd => {
                let amount: usize = cmd.as_str().parse().unwrap_or(1);
                let mut start = cursor.calculate_start_of_line(COLS);
                let mut end = start + (COLS * amount);
                start = cmp::min(start, buf.len());
                end = cmp::min(end, buf.len());
                buf.drain(start..end);
                // Move cursor if it is out of data
                cursor.trim_to_max_minus_one(buf.len());
                lastcommand = command.clone();
            }
            Rule::bigd => {
                let mut start = cursor.pos();
                // One more as we also want to delete the last character
                let mut end = cursor.calculate_end_of_line(COLS) + 1;
                start = cmp::min(start, buf.len());
                end = cmp::min(end, buf.len());
                buf.drain(start..end);
                // Move cursor if it is out of data
                cursor.trim_to_max_minus_one(buf.len());
                lastcommand = command.clone();
            }
            Rule::insert => {
                // The next chars will be inserted
                clear = false;
            }
            Rule::insertstuff => {
                let key = command.chars().last().unwrap_or('x');

                match cursor.selects() {
                    CursorSelects::LeftNibble => {
                        if let Some(c) = key.to_digit(16) {
                            buf.insert(cursor.pos(), (c as u8) << 4);
                            cursor.select_right_nibble();
                        }
                    }
                    CursorSelects::RightNibble => {
                        // This if checks if we are out of range already
                        if cursor.pos() == buf.len() {
                            // Then just insert some data
                            buf.insert(cursor.pos(), 0);
                        }
                        if let Some(c) = key.to_digit(16) {
                            buf[cursor.pos()] = buf[cursor.pos()] & 0xF0 | c as u8;
                            cursor.select_left_nibble();
                            // This puts the cursor out of range intentionally,
                            // inserting nibbles would feel strange otherwise.
                            cursor.add(1, buf.len() + 1);
                        }
                    }
                    CursorSelects::AsciiChar => {
                        buf.insert(cursor.pos(), key as u8);
                        // This puts the cursor out of range intentionally,
                        // this is probably later used by the command 'a'
                        cursor.add(1, buf.len() + 1);
                    }
                }

                clear = false;
            }
            Rule::insertend => {
                lastcommand = command.clone();
            }
            Rule::jumpascii => {
                cursor.swap_selection_hex_ascii();
            }
            Rule::querry => {
                // Most likely will be changed later
                infotext.push_str(&format!("Current byte marked: {}", cursor.pos()));
            }
            Rule::repeat => {
                autoparse = lastcommand.clone();
            }
            Rule::searchend => {
                if cursor.is_over_ascii() {
                    let searchstr = cmd.clone().into_inner().as_str();
                    let search = searchstr.as_bytes();
                    let newpos = match TwoWaySearcher::new(search).search_in(&buf) {
                        Some(t) => t,
                        None => {
                            infotext.push_str(&format!("Pattern not found: {}", searchstr));
                            cursor.pos() // Return same position
                        }
                    };
                    cursor.set_pos(newpos);
                } else {
                    infotext.push_str("Ascii-search works, when the cursor is over ascii");
                    command.pop();
                    clear = false;
                }
            }
            Rule::hexsearchend => {
                if cursor.is_over_ascii() {
                    let searchstr = cmd.clone().into_inner().as_str();
                    let search = searchstr.as_bytes();
                    let newpos = match TwoWaySearcher::new(search).search_in(&buf) {
                        Some(t) => t,
                        None => {
                            infotext.push_str(&format!("Pattern not found: {}", searchstr));
                            cursor.pos() // Return same position
                        }
                    };
                    cursor.set_pos(newpos);
                } else {
                    let searchstr = cmd.clone().into_inner().as_str();
                    let search = searchstr.as_bytes();
                    let newpos = match buf.search(search) {
                        Some(t) => t,
                        None => {
                            infotext.push_str(&format!("Pattern not found: {}", searchstr));
                            cursor.pos() // Return same position
                        }
                    };
                    cursor.set_pos(newpos);
                }
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
            Rule::gatherall => {
                // When the command is still to be fully built
                clear = false;
            }
            // The parser generates those cases, even when marked
            // as silent in src/cmd.pest (by using the "_"-prefix)
            // Maybe this could be fixed upstream?
            Rule::escape
            | Rule::movement
            | Rule::search
            | Rule::searchstr
            | Rule::hex_digit
            | Rule::quickstuffescaped
            | Rule::quickstuff
            | Rule::escape_char
            | Rule::anything_but_escape
            | Rule::backspace_char
            | Rule::cmd
            | Rule::gatherone
            | Rule::cmd_list => (),
        }

        if save {
            if path.exists() {
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(path)
                    .context("File could not be opened.")?;
                file.seek(SeekFrom::Start(0))?;
                file.write_all(&buf)?;
                file.set_len(buf.len() as u64)?;
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
        if cursor.pos() > ((screenheight + screenoffset).saturating_sub(1) * COLS).saturating_sub(1)
        {
            screenoffset = 2 + cursor.pos() / COLS - screenheight;
        }
        if cursor.pos() < screenoffset * COLS {
            screenoffset = cursor.pos() / COLS;
        }

        draw(&buf, COLS, &command, &infotext, cursor, screenoffset)?;
    }

    disable_raw_mode()?;
    Ok(())
}
