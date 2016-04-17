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

extern crate ncurses;
use ncurses::*;

//extern crate getopts;
//use getopts::Options;

fn main() {
    let mut buffer = vec![];
    let mut cursorpos:usize = 0;
    let mut cursorstate:usize = 0; //0 is left nibble, 1 is right nibble, 2 is ascii
    let mut screenoffset:usize = 0; // 0 = display data from first line of file
    const SPALTEN:usize = 16;
    let mut command = String::new();

    initscr(); //start ncursesw
    let screenheight : usize = getmaxy(stdscr) as usize;
    cbreak();  //ctrl+z and fg works with this
    noecho();
    start_color();
    init_pair(1, COLOR_GREEN, COLOR_BLACK);

    printw("Welcome to Hexdino.\nPlease find the any-key on your keyboard and press it.\n");

    let args: Vec<_> = env::args().collect();
    /*
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("f", "", "set file name", "NAME");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        let brief = format!("Usage: {} FILE [options]", program);
            print!("{}", opts.usage(&brief));
        return;
    }
    let path = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        let brief = format!("Usage: {} FILE [options]", program);
            print!("{}", opts.usage(&brief));
        return;
    };*/

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

//TODO in this order: 1)disp files 2) hjkl movement 3) edit file by 'r' 4) save file 5) edit file by 'x, i' 6) search by '/'

    let mut file = match OpenOptions::new().read(true).write(true).create(true).open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
            Error::description(&why)),
        Ok(file) => file,
    };

    file.read_to_end(&mut buffer).ok().expect("File could not be read.");

    let mut mode = 0; // 0 Command mode, 1 replace next char, 2 type a command, 3 Insert

    draw(&buffer, cursorpos, SPALTEN, screenheight, mode, &command, cursorstate, screenoffset);

    let mut key;
//    key = getch();
//    printw(&format!("{:?}", key));

    let mut ragequitnow = 0;
    while ragequitnow == 0 {
        key = getch();
        if mode == 0 { // movement mode
            match key {
                104 => { // Button "h"
                    if cursorstate == 2 { // ascii mode
                        if cursorpos > 0 { // not at start
                            cursorpos-=1; // go left
                        }
                    }
                    else if cursorstate == 1 { // hex mode, right nibble
                        cursorstate = 0; // left nibble
                    }
                    else if cursorstate == 0 { // hex mode, left nibble
                        if cursorpos > 0 { // not at start
                            cursorstate = 1; // right nibble
                            cursorpos-=1; // go left
                        }
                    }
                },
                106 => { //Button "j"
                    if cursorpos+SPALTEN < buffer.len() { // not at end
                        cursorpos+=SPALTEN; // go down
                    }
                    else { // when at end
                        cursorpos=buffer.len()-1; // go to end
                    }
                },
                107 => { //Button "k"
                    if cursorpos >= SPALTEN {
                        cursorpos-=SPALTEN;
                    }
                },
                108 => { // Button "l"
                    if cursorstate == 2 { // ascii mode
                        if cursorpos < buffer.len()-1 { // not at end
                            cursorpos+=1; // go right
                        }
                    }
                    else if cursorstate == 0 { // hex mode, left nibble
                        cursorstate = 1; // right nibble
                    }
                    else if cursorstate == 1 { // hex mode, right nibble
                        if cursorpos < buffer.len()-1 { // not at end
                            cursorstate = 0; // left nibble
                            cursorpos+=1; // go right
                        }
                    }
                },
                 48 => { // Button "0"
                    cursorpos -= cursorpos%SPALTEN; // jump to start of line
                    if cursorstate == 1 { // hex mode, right nibble
                        cursorstate = 0; // left nibble
                    }
                },
                 36 => { // Button "$"
                    if cursorpos-(cursorpos%SPALTEN)+(SPALTEN-1) < buffer.len() { // check if no overflow
                        cursorpos = cursorpos-(cursorpos%SPALTEN)+(SPALTEN-1); // jump to end of line
                    }
                    else {
                        cursorpos = buffer.len()-1 // jump to end of line
                    }
                    if cursorstate == 0 { // hex mode, left nibble
                        cursorstate = 1; // right nibble
                    }
                },
                114 => { // Button "r"
                    mode = 1; // replaces the next char
                },
                120 => { // Button "x"
                    if buffer.len() > 0 { // remove the next char
                        buffer.remove(cursorpos);
                    }
                },
                105 => { // Button "i"
                    mode = 3; // go to insert mode
                },
                 58 => { // Button ":"
                     mode = 2; // write a command
                     command.clear(); // delete old command
                 },
                 74 => { // Button "J"
                     if cursorstate == 2 { // ascii mode
                         cursorstate = 0; // hex mode, left nibble
                     }
                     else {
                         cursorstate = 2; // jump to ascii
                     }
                 },
//                 63 => printw("{:?}", asdf), //TODO: print available key helpfile
                _ => (),
            }

            //Always move screen when cursor leaves screen
            if cursorpos > (screenheight+screenoffset-1)*SPALTEN -1 {
                screenoffset+=1; // move screen
            }
            if cursorpos < screenoffset*SPALTEN {
                screenoffset-=1; // move screen
            }

        } else
        if mode == 1 && cursorstate == 2 { // r was pressed so replace next char in ascii mode
            match key {
                c @ 32...126 => { buffer[cursorpos] = c as u8 },
                _ => (),
            }
            mode = 0;
        } else
        if mode == 1 && cursorstate == 0 {
            match key { // Change left nibble
                c @ 65...70 | c @ 97...102 => // A-F or a-f
                    { buffer[cursorpos] = buffer[cursorpos]&0x0F | (c as u8 - 55)<<4 },
                c @ 48... 57 => // 0-9
                    { buffer[cursorpos] = buffer[cursorpos]&0x0F | (c as u8 - 48)<<4 },
                _ => ()
            }
            mode = 0;
        } else
        if mode == 1 && cursorstate == 1 {
            match key { // Change right nibble
                c @ 65...70 | c @ 97...102 => // A-F or a-f
                    { buffer[cursorpos] = buffer[cursorpos]&0xF0 | (c as u8 - 55) },
                c @ 48... 57 => // 0-9
                    { buffer[cursorpos] = buffer[cursorpos]&0xF0 | (c as u8 - 48) },
                _ => ()
            }
            mode = 0;
        } else
        if mode == 2 {
            match key {
                c @ 32...126 => { command.push(c as u8 as char); },
                27 => {command.clear();mode = 0;},
                10 => { // Enter pressed, compute command!
                        if (command == "w".to_string()) || (command == "wq".to_string()) {
                            file.seek(SeekFrom::Start(0)).ok().expect("Filepointer could not be set to 0");
                            file.write_all(&mut buffer).ok().expect("File could not be written.");
                            if command == "wq".to_string() {
                                ragequitnow = 1;
                            }
                            command.clear();mode = 0;
                        }
                        else if command == "q".to_string() {
                            ragequitnow = 1;
                        }
                        else {
                            command.clear();
                            command.push_str("Bad_command!");
                            mode = 0;
                        }
                    },
                _ => (),
            }
        } else
        if mode == 3 {
            if cursorstate == 0 { // Left nibble
                match key {
                    c @ 65...70 | c @ 97...102 => // A-F or a-f
                        {buffer.insert(cursorpos, (c as u8 - 55)<<4 ); cursorstate = 1;},
                    c @ 48... 57 => // 0-9
                        {buffer.insert(cursorpos, (c as u8 - 48)<<4); cursorstate = 1;},
                    27 => {mode = 0;},
                    _ => ()
                }
            } else
            if cursorstate == 1 { // Right nibble
                match key {
                    c @ 65...70 | c @ 97...102 => // A-F or a-f
                        { buffer[cursorpos] = buffer[cursorpos]&0xF0 | (c as u8 - 55); cursorstate = 0; cursorpos+=1; },
                    c @ 48...57 => // 0-9
                        { buffer[cursorpos] = buffer[cursorpos]&0xF0 | (c as u8 - 48); cursorstate = 0; cursorpos+=1; },
                    27 => {mode = 0;},
                    _ => ()
                }
            } else
            if cursorstate == 2 { // Ascii
                match key {
                    c @ 32...126 => { buffer.insert(cursorpos, c as u8); cursorpos+=1; },
                    27 => {mode = 0;},
                    _ => (),
                }
            }
        }
        draw(&buffer, cursorpos, SPALTEN, screenheight, mode, &command, cursorstate, screenoffset);
    }

    refresh();
    endwin();
}

fn draw(buffer:&Vec<u8>, cursorpos:usize, spalten:usize, maxzeilen:usize, mode:usize, command:&String, cursorstate:usize, screenoffset:usize) {
    erase();

    let mut zeilen = buffer.len() / spalten;
    if zeilen >= maxzeilen-1 { // Last line reserved for Status/Commands/etc (Like in vim)
        zeilen = maxzeilen-2;
    }

    for z in 0 .. zeilen+1 {
        printw(&format!("{:08X}: ", z*spalten+screenoffset*spalten )); // 8 hex digits (4GB/spalten or 0.25GB@spalten=SPALTEN)
        printw(" "); // Additional space between line number and hex
        for s in 0 .. spalten {
            if z*spalten+screenoffset*spalten+s < buffer.len() {
                if z*spalten+screenoffset*spalten+s == cursorpos { // Color of left nibble
                    if cursorstate == 0 {attron(COLOR_PAIR(1) | A_STANDOUT());}
                    else if cursorstate == 2 {attron(A_UNDERLINE());}
                }
                printw(&format!("{:01X}", buffer[z*spalten+screenoffset*spalten+s]>>4) ); // Display left nibble
                if z*spalten+screenoffset*spalten+s == cursorpos { // End of color left nibble
                    if cursorstate == 0 {attroff(COLOR_PAIR(1) | A_STANDOUT());}
                    else if cursorstate == 2 {attroff(A_UNDERLINE());}
                }

                if z*spalten+screenoffset*spalten+s == cursorpos { // Color of right nibble
                    if cursorstate == 1 {attron(COLOR_PAIR(1) | A_STANDOUT());}
                    else if cursorstate == 2 {attron(A_UNDERLINE());}
                }
                printw(&format!("{:01X}", buffer[z*spalten+screenoffset*spalten+s]&0x0F) ); // Display right nibble
                if z*spalten+screenoffset*spalten+s == cursorpos {
                    if cursorstate == 1 {attroff(COLOR_PAIR(1) | A_STANDOUT());}
                    else if cursorstate == 2 {attroff(A_UNDERLINE());}
                }
                printw(" ");
            } else {
                printw("-- ");
            }
        }
        printw(" "); // Additional space between hex and ascii
        for s in 0 .. spalten {
            if z*spalten+screenoffset*spalten+s == cursorpos {
                if cursorstate == 2 {attron(COLOR_PAIR(1) | A_STANDOUT());}
                else {attron(A_UNDERLINE());}
            }
            if z*spalten+screenoffset*spalten+s < buffer.len() {
                if let c @ 32...126 = buffer[z*spalten+screenoffset*spalten+s] {
                    if c as char == '%' {
                        printw("%%"); // '%' needs to be escaped by a '%' in ncurses
                    } else {
                        printw(&format!("{}", c as char) );
                    }
                }
                else {printw(&format!(".") );}
            }
            if z*spalten+screenoffset*spalten+s == cursorpos {
                if cursorstate == 2 {attroff(COLOR_PAIR(1) | A_STANDOUT());}
                else {attroff(A_UNDERLINE());}
            }
        }
        printw("\n");
    }
    for _ in 0 .. maxzeilen-zeilen-2 {
        printw("\n"); // Put the cursor on last line of terminal
    }
    if mode == 2 {
        printw(":"); // Indicate that a command can be typed in
    }
    if mode == 3 {
        printw("insert"); // Indicate that insert mode is active
    }
    printw(&format!("{}", command));
}
