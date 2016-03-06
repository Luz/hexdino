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
    const SPALTEN:usize = 16;
//    let mut command = vec![]; // If command is typed by user, store it
//    command.push(":asdf");
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
        printw(&format!("Opening file {} (TODO)", args[1]));
        args[1].clone()
    } else {
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
//        Err(why) => printw("oh... file not opened..."),
        Err(why) => panic!("couldn't open {}: {}", display,
            Error::description(&why)),
        Ok(file) => file,
    };

//    file.read_to_end(&mut buffer).ok().expect("File could not be read.");
    file.read_to_end(&mut buffer).ok().expect("File could not be read.");

    let mut mode = 0; // 0 Command mode, 1 replace next char, 2 type a command, TODO 3 Insert

    draw(&buffer, cursorpos, SPALTEN, screenheight, mode, &command);

    let mut key;
//    key = getch();
//    printw(&format!("{:?}", key));

    let mut ragequitnow = 0;
    while ragequitnow == 0 {
        key = getch();
        if mode == 0 { // movement mode
            match key {
                104 => if cursorpos != 0 {cursorpos-=1}, //left is "-1"
                106 => if cursorpos+SPALTEN < buffer.len() {cursorpos+=SPALTEN} //down is "+16"
                        else {cursorpos=buffer.len()-1}, //down if on last line is "to end"
                107 => if cursorpos >= SPALTEN {cursorpos-=SPALTEN}, //up is "-16"
                108 => if cursorpos != buffer.len()-1 {cursorpos+=1}, //right is "+1"
                 48 => cursorpos -= cursorpos%16, //start of line is "to start"
                 36 => if cursorpos-(cursorpos%16)+(SPALTEN-1) < buffer.len() {cursorpos = cursorpos-(cursorpos%16)+(SPALTEN-1)} //dollar is "to end"
                        else {cursorpos = buffer.len()-1},
                114 => mode = 1, //r replaces the next char
                 27 => ragequitnow = 1, // TODO replace by KEY_...?
                 58 => mode = 2, // ":" TODO use screenheight;
//                 63 => printw("{:?}", asdf), //TODO: print available key helpfile
                _ => (),
            }
        } else
        if mode == 1 { // r was pressed so replace next char
            match key {
                c @ 32...126 => { buffer[cursorpos] = c as u8; mode = 0 },
                27 => mode = 0,
                _ => (),
            }
        } else
        if mode == 2 {
            match key {
                c @ 32...126 => { command.push(c as u8 as char); }, //TODO check this
                27 => {command.clear();mode = 0;},
                10 => { // Enter pressed, compute command!
                        if command == "w".to_string() {
                            file.seek(SeekFrom::Start(0)).ok().expect("Filepointer could not be set to 0");
                            file.write_all(&mut buffer).ok().expect("File could not be written.");
                            mode = 0;
                        }
                    },
                _ => (),
            }
        }
 //       printw(&format!("{:?}", key));
        draw(&buffer, cursorpos, SPALTEN, screenheight, mode, &command);
    }

    refresh();
    endwin();
}

fn draw(buffer:&Vec<u8>, cursorpos:usize, spalten:usize, maxzeilen:usize, mode:usize, command:&String) {
//    let zeilen = buffer.len() / spalten;
    erase();

    let mut zeilen = buffer.len() / spalten;
    if zeilen >= maxzeilen-1 { // Last line reserved for Status/Commands/etc (Like in vim)
        zeilen = maxzeilen-2;
    }

    for z in 0 .. zeilen+1 {
        for s in 0 .. spalten {
            if z*spalten+s == cursorpos { attron(COLOR_PAIR(1)); }
            if z*spalten+s < buffer.len() { printw(&format!("{:02X} ", buffer[z*spalten+s]) ); }
                else {printw("-- "); }
            if z*spalten+s == cursorpos { attroff(COLOR_PAIR(1)); }
        }
        printw(" "); // Space between hex and ascii
        for s in 0 .. spalten {
            if z*spalten+s == cursorpos { attron(COLOR_PAIR(1)); }
                if z*spalten+s < buffer.len() {
                    if let c @ 32...126 = buffer[z*spalten+s] {printw(&format!("{}", c as char) );}
                    else {printw(&format!(".") );}
                }
            if z*spalten+s == cursorpos { attroff(COLOR_PAIR(1)); }
        }
        printw("\n");
    }
//TODO: use maxzeilen to draw the command (if in mode == 2) on last line of terminal
//show that the mode is 2 by adding a ':'
    if mode == 2 {
        printw(":");
    }
    printw(&format!("{}", command));
}
