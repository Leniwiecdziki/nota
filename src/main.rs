use carrot_libs::args;
mod input;
use crossterm::{terminal, cursor, execute};
use std::process;
use std::io::{self, Read};
use std::fs;
use std::str;
use std::collections::BTreeMap;

fn main() {
    println!("NOTA Text Editor");
    println!("Have fun!");
    println!();

    // Load options, switches and values
    let opts = args::opts();
    let (swcs, _vals) = args::swcs();

    // No options?
    if opts.is_empty() {
        eprintln!("No files to open!");
        process::exit(1);
    }
    // Some switches?
    if !swcs.is_empty() {
        eprintln!("This program does not accept any switches!");
        process::exit(1);
    }

    // Read every file and save it to "entire_file"
    for file in opts {
        println!("Opening file: {file}");
        match fs::File::open(file.clone()) {
            Err(e) => {
                eprintln!("{file}: Failed to open: {:?}!", e.kind());
                process::exit(1);
            },
            Ok(mut ret) => {
                let mut entire_file = String::new();
                ret.read_to_string(&mut entire_file).expect("{file}: Failed to perform read operation on file!");
                prepare(&mut entire_file);
            }
        }
    }
}

// Split file by lines
fn prepare(entire_file:&mut String) {
    // Collect some terminal info
    let terminal_width = terminal::size().unwrap().0 as usize;
    //let terminal_height = terminal::size().unwrap().1 as usize;

    // It contains the Line structs created below
    let mut lines = BTreeMap::new();

    // Cut line into parts if they are too long for the terminal window
    let mut index = 1;
    for line in entire_file.lines() {
        // If the line is longer than terminal window, save it's chunks separately
        if line.chars().count() > terminal_width {
            let subs = line.as_bytes().chunks(terminal_width).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();
            for s in subs {
                lines.insert(index, s.to_string());
                index+=1;
            }
        }
        // Otherwise, just throw entire line to line_parts
        else {
            lines.insert(index, line.to_string());
            index+=1;
        }
    }

    editor(lines, index);
}

fn editor(mut lines:BTreeMap<usize, String>, number_of_parts:usize) {
    // Collect some terminal info
    let terminal_width = terminal::size().unwrap().0 as usize;
    let terminal_height = terminal::size().unwrap().1 as usize;

    dbg!(&lines);

    terminal::enable_raw_mode().unwrap();
    execute!(io::stdout(), terminal::EnterAlternateScreen).unwrap();
    execute!(io::stdout(), cursor::MoveTo(0,0)).unwrap();

    // This defines from which line part the file should be printed
    // then, function printlines() would show lines until the end of terminal height is reached
    let mut start_from_line = 1;
    printlines(start_from_line, &lines);

    // Cursor position on the terminal
    execute!(io::stdout(), cursor::MoveTo(0,0)).unwrap();
    let mut curx = 0;
    let mut cury = 0;
    // Cursor index number in file
    let mut cur_on_line_in_file = 1;

    // Actions on keyboard keypress
    infobar(format!("Line: {cur_on_line_in_file}"), "status");
    loop {
        let (action, letter) = carrot_libs::input::detect();
        match (action,letter) {
            ("STOP", None) => break,
            ("UP", None) => {
                // Don't go below 0! That would violate our int and also 
                if cury > 0 {
                    cury -= 1;
                } else {
                    if start_from_line > 1 {
                        start_from_line -= 1;
                        printlines(start_from_line, &lines);
                    }
                }
                if cur_on_line_in_file > 1 {
                    cur_on_line_in_file-=1;
                }
                infobar(format!("Line: {cur_on_line_in_file}"), "status");
            },
            ("DOWN", None) => {
                if cury < terminal_height-2 {
                    cury+=1;
                } else {
                    if terminal_height+start_from_line+1 < number_of_parts {
                        start_from_line += 1;
                        printlines(start_from_line, &lines);
                    }
                }
                if cur_on_line_in_file < lines.len() {
                    cur_on_line_in_file+=1;
                }
                infobar(format!("Line: {cur_on_line_in_file}"), "status");
            },
            ("LEFT", None) => {
                if curx > 0 {
                    curx-=1;
                }
            },
            ("RIGHT", None) => {
                if curx < terminal_width-1 {
                    curx+=1;
                }
                infobar(format!("Line: {cur_on_line_in_file}"), "status");
            },
            ("CHAR", Some('i')) => {
                // This is self-explanatory
                infobar("Insert mode".to_string(), "neutral");
                execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)).unwrap();
                print!("\r");
                // Save user input to 'user_input'
                let user_input = input::get("".to_string(), Some(lines[&cur_on_line_in_file].clone()), curx);
                // If user_input is not wider than terminal, just replace current line part
                // if it is wider - replace current line part and add new ones.
                if user_input.len() < terminal_height {
                    lines.get_mut(&cur_on_line_in_file).map(|val| { *val = user_input; });
                }

                execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)).unwrap();
                printlines(start_from_line, &lines);
                infobar(format!("Normal mode"), "neutral");
            },
            _ => (),
        }
        execute!(io::stdout(), cursor::MoveTo(curx as u16, cury as u16)).unwrap();
    }
    terminal::disable_raw_mode().unwrap();
    execute!(io::stdout(), terminal::LeaveAlternateScreen).unwrap();
}

fn printlines(start:usize, lines:&BTreeMap<usize, String>) {
    let terminal_height = terminal::size().unwrap().1 as usize - 1;
    let mut idx = start;
    let end = terminal_height+start;
    execute!(io::stdout(), cursor::SavePosition).unwrap();
    execute!(io::stdout(), cursor::MoveTo(0, 0)).unwrap();
    execute!(io::stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
    while idx < end {
        match lines.get(&idx) {
            Some(e) => println!("{}\r", e),
            None => println!(),
        };
        idx+=1;
    }
    execute!(io::stdout(), cursor::RestorePosition).unwrap();
}

fn infobar(funny:String, color:&'static str) {
    execute!(io::stdout(), cursor::SavePosition).unwrap();
    let terminal_height = terminal::size().unwrap().1;
    execute!(io::stdout(), cursor::MoveTo(0, terminal_height)).unwrap();
    execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)).unwrap();
    match color {
        "status" => print!("\x1b[107m\x1b[30m{funny}\x1b[0m"), // White background, black text
        "warn" => print!("\x1b[103m\x1b[30m{funny}\x1b[0m"), // Yellow background, black text
        "error" => print!("\x1b[41m\x1b[97m{funny}\x1b[0m"), // Red background, white text
        "good" => print!("\x1b[42m\x1b[97m{funny}\x1b[0m"), // Green, white
        "neutral" => print!("\x1b[104m\x1b[97m{funny}\x1b[0m"), // Blue, white
        "critical" => print!("\x1b[102m\x1b[97m{funny}\x1b[0m"), // Purple, white
        _ => print!("{funny}"),
    }
    execute!(io::stdout(), cursor::RestorePosition).unwrap();
}
