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

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "cmd.pest"]
struct IdentParser;

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
    let screenheight = getmaxy(stdscr()) as usize;
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
        Ok(m) => m,
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
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
            Ok(file) => file,
        };
        file.read_to_end(&mut buf).ok().expect(
            "File could not be read.",
        );
    }

    draw(&buf, cursorpos, SPALTEN, &command, cstate, screenoffset);

    let mut quitnow = false;
    while quitnow == false {
        let key = std::char::from_u32(getch() as u32).unwrap();
        printw(&format!("   {:?}   ", key));
        command.push_str(&key.clone().to_string());

        let parsethisstring = command.clone();
        let commands = IdentParser::parse(Rule::cmd_list, &parsethisstring)
            .unwrap_or_else(|e| panic!("{}", e));

        let mut clear = true;
        let mut save = false;
        for cmd in commands {
            match cmd.as_rule() {
                Rule::down => {
                    printw(&format!("{:?}", cmd.as_rule()));
                    if cursorpos + SPALTEN < buf.len() {
                        // not at end
                        cursorpos += SPALTEN;
                    } else {
                        // when at end
                        if buf.len() != 0 {
                            // Suppress underflow
                            cursorpos = buf.len() - 1;
                        }
                    }
                }
                Rule::up => {
                    if cursorpos >= SPALTEN {
                        cursorpos -= SPALTEN;
                    }
                }
                Rule::left => {
                    if cstate == Cursorstate::Asciichar {
                        if cursorpos > 0 {
                            cursorpos -= 1;
                        }
                    } else if cstate == Cursorstate::Rightnibble {
                        cstate = Cursorstate::Leftnibble;
                    } else if cstate == Cursorstate::Leftnibble {
                        if cursorpos > 0 {
                            // not at start
                            cstate = Cursorstate::Rightnibble;
                            cursorpos -= 1;
                        }
                    }
                }
                Rule::right => {
                    if cstate == Cursorstate::Asciichar {
                        if cursorpos + 1 < buf.len() {
                            // not at end
                            cursorpos += 1;
                        }
                    } else if cstate == Cursorstate::Leftnibble {
                        cstate = Cursorstate::Rightnibble;
                    } else if cstate == Cursorstate::Rightnibble {
                        if cursorpos + 1 < buf.len() {
                            // not at end
                            cstate = Cursorstate::Leftnibble;
                            cursorpos += 1;
                        }
                    }
                }
                Rule::start => {
                    cursorpos -= cursorpos % SPALTEN; // jump to start of line
                    if cstate == Cursorstate::Rightnibble {
                        cstate = Cursorstate::Leftnibble;
                    }
                }
                Rule::end => {
                    // check if no overflow
                    if cursorpos - (cursorpos % SPALTEN) + (SPALTEN - 1) < buf.len() {
                        // jump to end of line
                        cursorpos = cursorpos - (cursorpos % SPALTEN) + (SPALTEN - 1);
                    } else {
                        // jump to end of line
                        cursorpos = buf.len() - 1
                    }
                    if cstate == Cursorstate::Leftnibble {
                        cstate = Cursorstate::Rightnibble;
                    }
                }
                Rule::replace => {
                    // printw("next char will be the replacement!");
                    clear = false;
                }
                Rule::remove => {
                    if buf.len() > 0 {
                        // remove the current char
                        buf.remove(cursorpos);
                        if cursorpos >= buf.len() && cursorpos > 0 {
                            cursorpos -= 1;
                        }
                    }
                }
                Rule::insert => {
                    printw("next chars will be inserted!");
                    clear = false;
                }
                Rule::jumpascii => {
                    if cstate == Cursorstate::Asciichar {
                        cstate = Cursorstate::Leftnibble;
                    } else {
                        cstate = Cursorstate::Asciichar;
                    }
                }
                Rule::helpfile => {
                    command.push_str("No helpfile yet");
                }
                Rule::search => (),
                Rule::backspace => {
                    command.pop();
                    command.pop();
                    clear = false;
                },
                _ => (),
            }

            for inner_cmd in cmd.into_inner() {
                match inner_cmd.as_rule() {
                    Rule::replacement => {
                        // TODO: use inner_cmd and not just "key"
                        // printw(&format!("Replacement: {:?}", inner_cmd.as_str()));
                        if cstate == Cursorstate::Asciichar {
                            if cursorpos >= buf.len() {
                                buf.insert(cursorpos, 0);
                            }
                            // buf[cursorpos] = inner_cmd.as_str();
                            buf[cursorpos] = key as u8;
                        } else {
                            let mask = if cstate == Cursorstate::Leftnibble {
                                0x0F
                            } else {
                                0xF0
                            };
                            let shift = if cstate == Cursorstate::Leftnibble {
                                4
                            } else {
                                0
                            };
                            if cursorpos >= buf.len() {
                                buf.insert(cursorpos, 0);
                            }
                            // Change the selected nibble
                            if let Some(c) = key.to_digit(16) {
                                buf[cursorpos] = buf[cursorpos] & mask | (c as u8) << shift;
                            }
                        }

                    }
                    // TODO: use inner_cmd and not just "key"
                    Rule::insertment => {
                        // printw(&format!("Inserted: {:?}", inner_cmd.as_str()));
                        command.pop(); // remove the just inserted thing
                        clear = false;

                        if cstate == Cursorstate::Leftnibble {
                            // Left nibble
                            if let Some(c) = (key as char).to_digit(16) {
                                buf.insert(cursorpos, (c as u8) << 4);
                                cstate = Cursorstate::Rightnibble;
                            }
                        } else if cstate == Cursorstate::Rightnibble {
                            // Right nibble
                            if cursorpos == buf.len() {
                                buf.insert(cursorpos, 0);
                            }
                            if let Some(c) = (key as char).to_digit(16) {
                                buf[cursorpos] = buf[cursorpos] & 0xF0 | c as u8;
                                cstate = Cursorstate::Leftnibble;
                                cursorpos += 1;
                            }
                        } else if cstate == Cursorstate::Asciichar {
                            buf.insert(cursorpos, key as u8);
                            cursorpos += 1;
                        }
                    }
                    Rule::searchstr => {
                        // printw(&format!("Searching for: {:?}", inner_cmd.as_str() ))
                        let search = inner_cmd.as_str().as_bytes();
                        cursorpos = buf.find_subset(&search).unwrap_or(cursorpos);
                    }
                    Rule::saveandexit => {
                        save = true;
                        quitnow = true;
                    }
                    Rule::exit => quitnow = true,
                    Rule::save => save = true,
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
                        .open(&path) {
                        Err(why) => {
                            panic!("couldn't open {}: {}", display, Error::description(&why))
                        }
                        Ok(file) => file,
                    };
                    file.seek(SeekFrom::Start(0)).ok().expect(
                        "Filepointer could not be set to 0",
                    );
                    file.write_all(&mut buf).ok().expect(
                        "File could not be written.",
                    );
                    file.set_len(buf.len() as u64).ok().expect(
                        "File could not be set to correct lenght.",
                    );
                    command.push_str("File saved!");
                } else {
                    command.push_str("path.exists() failed");
                }
                // TODO: define filename during runtime
                save = false;
            }
            if clear {
                command.clear();
            }

            // Always move screen when cursor leaves screen
            if cursorpos > (screenheight + screenoffset - 1) * SPALTEN - 1 {
                screenoffset += 1; // move screen
            }
            if cursorpos < screenoffset * SPALTEN {
                screenoffset -= 1; // move screen
            }


        }

        draw(&buf, cursorpos, SPALTEN, &command, cstate, screenoffset);
    }

    refresh();
    endwin();
}
