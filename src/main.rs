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

mod find;
use find::*;
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
use cursor::Cursor;

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
        .open(&path)
        .context("File could not be opened.")?;

    file.read_to_end(&mut buf).expect("File could not be read.");

    enable_raw_mode()?;
    draw(&buf, COLS, &command, &mut infotext, cursor, screenoffset)?;

    let mut quitnow = false;
    while quitnow == false {
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

        // Info can always be cleared as soon as there is a new input
        infotext.clear();

        match cmd.as_rule() {
            Rule::down => {
                cursor.add(COLS, buf.len());
            }
            Rule::up => {
                cursor.sub(COLS, 0);
            }
            Rule::left => {
                if cursor.is_over_ascii() {
                    cursor.sub(1, 0);
                } else if cursor.is_over_right_nibble() {
                    cursor.select_left_nibble();
                } else if cursor.is_over_left_nibble() {
                    if cursor.pos() >= 1 {
                        // not at start
                        cursor.select_right_nibble();
                    }
                    cursor.sub(1, 0);
                }
            }
            Rule::right => {
                if cursor.is_over_ascii() {
                    cursor.add(1, buf.len());
                } else if cursor.is_over_left_nibble() {
                    cursor.select_right_nibble();
                } else if cursor.is_over_right_nibble() {
                    if cursor.pos() < buf.len().saturating_sub(1) {
                        // not at end
                        cursor.select_left_nibble();
                    }
                    cursor.add(1, buf.len());
                }
            }
            Rule::start => {
                cursor.jump_to_start_of_line(COLS);
                if cursor.is_over_right_nibble() {
                    cursor.select_left_nibble();
                }
            }
            Rule::end => {
                cursor.jump_to_end_of_line(COLS, buf.len());
                if cursor.is_over_left_nibble() {
                    cursor.select_right_nibble();
                }
            }
            Rule::bottom => {
                let pos_on_line = cursor.calculate_pos_on_line(COLS);
                let line = buf.len().saturating_sub(1) / COLS;
                cursor.jump_to_pos_on_line(line, pos_on_line, COLS, buf.len());
            }
            Rule::replace => {
                clear = false;
            }
            Rule::replacement => {
                let key = command.chars().last().unwrap_or('x');
                if cursor.is_over_ascii() {
                    if cursor.pos() >= buf.len() {
                        buf.insert(cursor.pos(), 0);
                    }
                    buf[cursor.pos()] = key as u8;
                } else {
                    let mask = if cursor.is_over_left_nibble() {
                        0x0F
                    } else {
                        0xF0
                    };
                    let shift = if cursor.is_over_left_nibble() { 4 } else { 0 };
                    if cursor.pos() >= buf.len() {
                        buf.insert(cursor.pos(), 0);
                    }
                    // Change the selected nibble
                    if let Some(c) = key.to_digit(16) {
                        buf[cursor.pos()] = buf[cursor.pos()] & mask | (c as u8) << shift;
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
                if buf.len() > 0 && cursor.pos() < buf.len() {
                    // remove the current char
                    buf.remove(cursor.pos());
                }
                // Move cursor if it is out of data
                cursor.trim_to_max_minus_one(buf.len());
                lastcommand = command.clone();
            }
            Rule::dd => {
                let amount: usize = cmd.as_str().parse().unwrap_or(1);
                if cursor.pos() < buf.len() {
                    let startofline = cursor.calculate_start_of_line(COLS);
                    let mut endofline = startofline + (COLS * amount);
                    endofline = cmp::min(endofline, buf.len());
                    buf.drain(startofline..endofline);
                    cursor.trim_to_max_minus_one(buf.len());
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

                if cursor.is_over_left_nibble() {
                    if let Some(c) = key.to_digit(16) {
                        buf.insert(cursor.pos(), (c as u8) << 4);
                        cursor.select_right_nibble();
                    }
                } else if cursor.is_over_right_nibble() {
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
                } else if cursor.is_over_ascii() {
                    buf.insert(cursor.pos(), key as u8);
                    // This puts the cursor out of range intentionally,
                    // this is probably later used by the command 'a'
                    cursor.add(1, buf.len() + 1);
                }

                clear = false;
            }
            Rule::insertend => {
                lastcommand = command.clone();
                clear = true;
            }
            Rule::jumpascii => {
                if cursor.is_over_ascii() {
                    cursor.select_left_nibble();
                } else {
                    cursor.select_ascii();
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
                let line: usize = cmd.as_str().parse().unwrap_or(0);
                cursor.jump_to_line(line, COLS, buf.len());
                clear = true;
            }
            Rule::searchend => {
                let searchstr = cmd.clone().into_inner().as_str();
                let search = searchstr.as_bytes();
                let foundpos = TwoWaySearcher::new(&search);
                cursor.set_pos(foundpos.search_in(&buf).unwrap_or(cursor.pos()));
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
                cursor.set_pos(buf.find_subset(&needle).unwrap_or(cursor.pos()));
                // infotext.push_str(&format!("Searching for: {:?}", needle ));
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
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .context("File could not be opened.")?;
                file.seek(SeekFrom::Start(0))
                    .expect("Filepointer could not be set to 0");
                file.write_all(&mut buf)
                    .expect("File could not be written.");
                file.set_len(buf.len() as u64)
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
        if cursor.pos() > (screenheight + screenoffset - 1) * COLS - 1 {
            screenoffset = 2 + cursor.pos() / COLS - screenheight;
        }
        if cursor.pos() < screenoffset * COLS {
            screenoffset = cursor.pos() / COLS;
        }

        draw(&buf, COLS, &command, &mut infotext, cursor, screenoffset)?;
    }

    disable_raw_mode()?;
    Ok(())
}
