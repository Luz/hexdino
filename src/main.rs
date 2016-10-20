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

// extern crate getopts;
// use getopts::Options;

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

fn main() {
    let mut buf = vec![];
    let mut cursorpos: usize = 0;
    // 0 is left nibble, 1 is right nibble, 2 is ascii
    let mut cursorstate: usize = 0;
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

    printw("Welcome to Hexdino.\nPlease find the any-key on your keyboard and press it.\n");

    let args: Vec<_> = env::args().collect();
    // let program = args[0].clone();
    // let mut opts = Options::new();
    // opts.optopt("f", "", "set file name", "NAME");
    // opts.optflag("h", "help", "print this help menu");
    //
    // let matches = match opts.parse(&args[1..]) {
    // Ok(m) => { m }
    // Err(f) => { panic!(f.to_string()) }
    // };
    // if matches.opt_present("h") {
    // let brief = format!("Usage: {} FILE [options]", program);
    // print!("{}", opts.usage(&brief));
    // return;
    // }
    // let path = if !matches.free.is_empty() {
    // matches.free[0].clone()
    // } else {
    // let brief = format!("Usage: {} FILE [options]", program);
    // print!("{}", opts.usage(&brief));
    // return;
    // };

    let path = if args.len() > 1 {
        printw(&format!("Opening file {}", args[1]));
        args[1].clone()
    } else {
        printw(&format!("No file specified. Trying to open foo.txt"));
        "foo.txt".into()
    };

    let path = Path::new(&path);
    let display = path.display();

    getch();
    clear();

    if !has_colors() {
        endwin();
        println!("Your terminal does not support color!\n");
        return;
    }

    // TODO in this order: 1)disp files 2) hjkl movement 3) edit file by 'r' 4) save file 5) edit
    // file by 'x, i' 6) search by '/'

    let mut file = match OpenOptions::new().read(true).write(true).create(true).open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    file.read_to_end(&mut buf).ok().expect("File could not be read.");

    let mut mode = Mode::Command;

    draw(&buf,
         cursorpos,
         SPALTEN,
         mode,
         &command,
         cursorstate,
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
                    if cursorstate == 2 {
                        // ascii mode
                        if cursorpos > 0 {
                            // not at start
                            cursorpos -= 1; // go left
                        }
                    } else if cursorstate == 1 {
                        // hex mode, right nibble
                        cursorstate = 0; // left nibble
                    } else if cursorstate == 0 {
                        // hex mode, left nibble
                        if cursorpos > 0 {
                            // not at start
                            cursorstate = 1; // right nibble
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
                    if cursorstate == 2 {
                        // ascii mode
                        if cursorpos < buf.len() - 1 {
                            // not at end
                            cursorpos += 1; // go right
                        }
                    } else if cursorstate == 0 {
                        // hex mode, left nibble
                        cursorstate = 1; // right nibble
                    } else if cursorstate == 1 {
                        // hex mode, right nibble
                        if cursorpos < buf.len() - 1 {
                            // not at end
                            cursorstate = 0; // left nibble
                            cursorpos += 1; // go right
                        }
                    }
                }
                '0' => {
                    cursorpos -= cursorpos % SPALTEN; // jump to start of line
                    if cursorstate == 1 {
                        // hex mode, right nibble
                        cursorstate = 0; // left nibble
                    }
                }
                '$' => {
                    if cursorpos - (cursorpos % SPALTEN) + (SPALTEN - 1) < buf.len() {
                        // check if no overflow
                        // jump to end of line
                        cursorpos = cursorpos - (cursorpos % SPALTEN) + (SPALTEN - 1);
                    } else {
                        cursorpos = buf.len() - 1 // jump to end of line
                    }
                    if cursorstate == 0 {
                        // hex mode, left nibble
                        cursorstate = 1; // right nibble
                    }
                }
                'r' => {
                    mode = Mode::Replace;
                }
                'x' => {
                    if buf.len() > 0 {
                        // remove the current char
                        buf.remove(cursorpos);
                        if cursorpos >= buf.len() {
                            cursorpos = buf.len() - 1;
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
                    if cursorstate == 2 {
                        // ascii mode
                        cursorstate = 0; // hex mode, left nibble
                    } else {
                        cursorstate = 2; // jump to ascii
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

        } else if mode == Mode::Replace && cursorstate == 2 {
            // r was pressed so replace next char in ascii mode
            match key {
                c @ 32...126 => buf[cursorpos] = c,
                _ => (),
            }
            mode = Mode::Command;
        } else if mode == Mode::Replace {
            let mask = if cursorstate == 0 { 0x0F } else { 0xF0 };
            let shift = if cursorstate == 0 { 4 } else { 0 };
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
                        file.seek(SeekFrom::Start(0))
                            .ok()
                            .expect("Filepointer could not be set to 0");
                        file.write_all(&mut buf).ok().expect("File could not be written.");
                        if command == "wq".to_string() {
                            quitnow = 1;
                        }
                        command.clear();
                        mode = Mode::Command;
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
            if cursorstate == 0 {
                // Left nibble
                match key as char {
                    c @ 'A'...'F' => {
                        buf.insert(cursorpos, (c as u8 - 55) << 4);
                        cursorstate = 1;
                    }
                    c @ 'a'...'f' => {
                        buf.insert(cursorpos, (c as u8 - 87) << 4);
                        cursorstate = 1;
                    }
                    c @ '0'...'9' => {
                        buf.insert(cursorpos, (c as u8 - 48) << 4);
                        cursorstate = 1;
                    }
                    '\u{1B}' => {
                        mode = Mode::Command;
                    }
                    _ => (),
                }
            } else if cursorstate == 1 {
                // Right nibble
                match key as char {
                    c @ 'A'...'F' => {
                        buf[cursorpos] = buf[cursorpos] & 0xF0 | (c as u8 - 55);
                        cursorstate = 0;
                        cursorpos += 1;
                    }
                    c @ 'a'...'f' => {
                        buf[cursorpos] = buf[cursorpos] & 0xF0 | (c as u8 - 87);
                        cursorstate = 0;
                        cursorpos += 1;
                    }
                    c @ '0'...'9' => {
                        buf[cursorpos] = buf[cursorpos] & 0xF0 | (c as u8 - 48);
                        cursorstate = 0;
                        cursorpos += 1;
                    }
                    '\u{1B}' => {
                        mode = Mode::Command;
                    }
                    _ => (),
                }
            } else if cursorstate == 2 {
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
            //            if cursorstate == 2 { // Ascii search
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
                    if cursorstate == 2 {
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
             cursorstate,
             screenoffset);
    }

    refresh();
    endwin();
}
