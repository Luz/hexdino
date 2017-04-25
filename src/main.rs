//! # Hexdino
//!
//! A hex editor with vim like keybindings written in Rust.

#![doc(html_logo_url = "https://raw.githubusercontent.com/Luz/hexdino/master/logo.png")]

use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io::SeekFrom;
use std::path::Path;
use std::error::Error;
use std::env;

mod draw;
mod find;
use draw::draw;
use find::FindSubset;

extern crate ncurses;
use ncurses::*;

extern crate getopts;
use getopts::Options;

#[derive(PartialEq, Copy, Clone)]
pub enum Mode {
    Command,
    Replace,
    TypeCommand,
    Insert,
    TypeSearch,
    SearchIt,
    SearchItHex,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Cursorstate {
    Leftnibble,
    Rightnibble,
    Asciichar,
}

fn main() {
    let mut buf = vec![];
    let mut cursorpos: usize = 0;
    let mut cstate: Cursorstate = Cursorstate::Leftnibble;
    // 0 = display data from first line of file
    let mut screenoffset: usize = 0;
    const SPALTEN: usize = 16;
    let mut command = String::new();

    // start ncursesw
    initscr();
    let screenheight: usize = getmaxy(stdscr) as usize;
    // ctrl+z and fg works with this
    cbreak();
    noecho();
    start_color();
    init_pair(1, COLOR_GREEN, COLOR_BLACK);

    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            println!("{}", f.to_string());
            println!("Usage: {} FILE [options]", program);
            endwin();
            return;
        }
    };
    if matches.opt_present("h") {
        println!("Usage: {} FILE [options]", program);
        endwin();
        return;
    }

    let patharg = if !matches.free.is_empty() {
        command.push_str(&format!("File {} was just opened", matches.free[0].clone()));
        matches.free[0].clone()
    } else {
        printw(&format!("Usage: {} FILE [options]", program));
        String::new()
    };

    let path = Path::new(&patharg);
    let display = path.display();

    if !has_colors() {
        endwin();
        println!("Your terminal does not support color!\n");
        return;
    }

    if patharg != "" {
        let mut file = match OpenOptions::new().read(true).write(true).create(true).open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
            Ok(file) => file,
        };
        file.read_to_end(&mut buf).ok().expect("File could not be read.");
    }

    let mut mode = Mode::Command;

    draw(&buf,
         cursorpos,
         SPALTEN,
         mode,
         &command,
         cstate,
         screenoffset);

    let mut key;
    //    key = getch() as u8;
    //    printw(&format!("{:?}", key));

    let mut quitnow = 0;
    while quitnow == 0 {
        key = getch() as u8;
        if mode == Mode::Command {
            match key as char {
                'h' => {
                    if cstate == Cursorstate::Asciichar {
                        // ascii mode
                        if cursorpos > 0 {
                            // not at start
                            cursorpos -= 1; // go left
                        }
                    } else if cstate == Cursorstate::Rightnibble {
                        cstate = Cursorstate::Leftnibble;
                    } else if cstate == Cursorstate::Leftnibble {
                        if cursorpos > 0 {
                            // not at start
                            cstate = Cursorstate::Rightnibble;
                            cursorpos -= 1; // go left
                        }
                    }
                }
                'j' => {
                    if cursorpos + SPALTEN < buf.len() {
                        // not at end
                        cursorpos += SPALTEN; // go down
                    } else {
                        // when at end
                        if buf.len() != 0 {
                            // Suppress underflow
                            cursorpos = buf.len() - 1; // go to end
                        }
                    }
                }
                'k' => {
                    if cursorpos >= SPALTEN {
                        cursorpos -= SPALTEN;
                    }
                }
                'l' => {
                    if cstate == Cursorstate::Asciichar {
                        // ascii mode
                        if cursorpos + 1 < buf.len() {
                            // not at end
                            cursorpos += 1; // go right
                        }
                    } else if cstate == Cursorstate::Leftnibble {
                        cstate = Cursorstate::Rightnibble;
                    } else if cstate == Cursorstate::Rightnibble {
                        if cursorpos + 1 < buf.len() {
                            // not at end
                            cstate = Cursorstate::Leftnibble;
                            cursorpos += 1; // go right
                        }
                    }
                }
                '0' => {
                    cursorpos -= cursorpos % SPALTEN; // jump to start of line
                    if cstate == Cursorstate::Rightnibble {
                        cstate = Cursorstate::Leftnibble;
                    }
                }
                '$' => {
                    if cursorpos - (cursorpos % SPALTEN) + (SPALTEN - 1) < buf.len() {
                        // check if no overflow
                        cursorpos = cursorpos - (cursorpos % SPALTEN) + (SPALTEN - 1); // jump to end of line
                    } else {
                        cursorpos = buf.len() - 1 // jump to end of line
                    }
                    if cstate == Cursorstate::Leftnibble {
                        cstate = Cursorstate::Rightnibble;
                    }
                }
                'r' => {
                    mode = Mode::Replace;
                }
                'x' => {
                    if buf.len() > 0 {
                        // remove the current char
                        buf.remove(cursorpos);
                        if cursorpos >= buf.len() && cursorpos > 0 {
                            cursorpos -= 1;
                        }
                    }
                }
                'i' => {
                    mode = Mode::Insert;
                }
                ':' => {
                    command.clear(); // delete old command
                    mode = Mode::TypeCommand;
                }
                'J' => {
                    if cstate == Cursorstate::Asciichar {
                        cstate = Cursorstate::Leftnibble;
                    } else {
                        cstate = Cursorstate::Asciichar;
                    }
                }
                '?' => {
                    // TODO
                    command.push_str("No helpfile yet");
                }
                '/' => {
                    // TODO
                    command.clear();
                    mode = Mode::TypeSearch;
                }
                _ => (),
            }

            // Always move screen when cursor leaves screen
            if cursorpos > (screenheight + screenoffset - 1) * SPALTEN - 1 {
                screenoffset += 1; // move screen
            }
            if cursorpos < screenoffset * SPALTEN {
                screenoffset -= 1; // move screen
            }

        } else if mode == Mode::Replace && cstate == Cursorstate::Asciichar {
            // r was pressed so replace next char in ascii mode
            if cursorpos >= buf.len() {
                buf.insert(cursorpos, 0 );
            }
            match key {
                c @ 32...126 => buf[cursorpos] = c,
                _ => (),
            }
            mode = Mode::Command;
        } else if mode == Mode::Replace {
            let mask = if cstate == Cursorstate::Leftnibble { 0x0F } else { 0xF0 };
            let shift = if cstate == Cursorstate::Leftnibble { 4 } else { 0 };
            if cursorpos >= buf.len() {
                buf.insert(cursorpos, 0 );
            }
            // Change the selected nibble
            if let Some(c) = (key as char).to_digit(16) {
                buf[cursorpos] = buf[cursorpos] & mask | (c as u8) << shift;
            }
            mode = Mode::Command;
        } else if mode == Mode::TypeCommand {
            match key {
                c @ 32...126 => {
                    command.push(c as char);
                }
                27 => {
                    command.clear();
                    mode = Mode::Command;
                }
                10 => {
                    // Enter pressed, compute command!
                    if (command == "w".to_string()) || (command == "wq".to_string()) {
                        if path.exists() {
                            let mut file = match OpenOptions::new().read(true).write(true).create(true).open(&path) {
                            Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
                            Ok(file) => file,
                            };
                            file.seek(SeekFrom::Start(0))
                                .ok()
                                .expect("Filepointer could not be set to 0");
                            file.write_all(&mut buf).ok().expect("File could not be written.");
                            file.set_len(buf.len() as u64).ok().expect("File could not be set to correct lenght.");
                            if command == "wq".to_string() {
                                quitnow = 1;
                            }
                        } else {
                            command.clear();
                            command.push_str("No filename specified! Not saved yet!");
//TODO: define filename during runtime
                        }
                        if command == "w".to_string() {
                            command.clear();
                            mode = Mode::Command;
                        }
                    } else if command == "q".to_string() {
                        quitnow = 1;
                    } else {
                        command.clear();
                        command.push_str("Bad_command!");
                        mode = Mode::Command;
                    }
                }
                _ => (),
            }
        } else if mode == Mode::Insert {
            if cstate == Cursorstate::Leftnibble {
                // Left nibble
                if let Some(c) = (key as char).to_digit(16) {
                    buf.insert(cursorpos, (c as u8) << 4); cstate = Cursorstate::Rightnibble;
                }
                if key == 27 {mode = Mode::Command;};
            } else if cstate == Cursorstate::Rightnibble {
                // Right nibble
                if cursorpos == buf.len() {
                    buf.insert(cursorpos, 0 );
                }
                if let Some(c) = (key as char).to_digit(16) {
                    buf[cursorpos] = buf[cursorpos]&0xF0 | c as u8; cstate = Cursorstate::Leftnibble; cursorpos+=1;
                }
                if key == 27 {mode = Mode::Command;};
            } else if cstate == Cursorstate::Asciichar {
                // Ascii
                match key {
                    c @ 32...126 => {
                        buf.insert(cursorpos, c);
                        cursorpos += 1;
                    }
                    27 => {
                        mode = Mode::Command;
                    }
                    _ => (),
                }
            }
        } else if mode == Mode::TypeSearch {
            //            if cstate == Cursorstate::Asciichar { // Ascii search
            match key {
                c @ 32...126 => {
                    command.push(c as char);
                }
                27 => {
                    command.clear();
                    mode = Mode::Command;
                }
                10 => {
                    // Enter pressed, compute command!
                    if cstate == Cursorstate::Asciichar {
                        mode = Mode::SearchIt;
                    } else {
                        mode = Mode::SearchItHex;
                    }
                }
                _ => (),
            }
            //            }

            if mode == Mode::SearchIt {
                let search = command.as_bytes();
                cursorpos = buf.find_subset(&search).unwrap_or(cursorpos);
                mode = Mode::Command;
                // command.push_str("Bad_command!");
            }
            if mode == Mode::SearchItHex {
                // TODO
                mode = Mode::Command;
                command.push_str("Hex search not yet implemented!");
            }
        }
        draw(&buf,
             cursorpos,
             SPALTEN,
             mode,
             &command,
             cstate,
             screenoffset);
    }

    refresh();
    endwin();
}
