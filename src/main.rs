use carrot_libs::{args, input};
use crossterm::{terminal, cursor, execute};
use std::process;
use std::io::{self, Read};
use std::fs;
use std::str;

// Type used for storing information about each file line
#[derive(Debug)]
struct Line {
    // It's number in file
    line_number: usize,
    // Number of parts that were created to fit in terminal
    parts_count: usize,
    // Parts!
    line_parts: Vec<Vec<String>>,
}

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
                prepare(entire_file);
            }
        }
    }
}

// Split file by lines
fn prepare(entire_file:String) {
    // Collect some terminal info
    let terminal_width = terminal::size().unwrap().0 as usize;
    let terminal_height = terminal::size().unwrap().1 as usize;

    // It contains the Line structs created below
    let mut lines_data = Vec::new();

    // Save file lines separately to a list
    // This contains parts of line shortened to fit into terminal
    // Example:
    // ["exa", "mpl", "e h", "ere"]
    let mut line_parts = Vec::new();
    // This thing is a list which stores another list of shortened file lines
    // Example:
    // [["exa", "mpl", "e h", "ere"], ["ano", "the", "r l", "ine"]]
    let mut lines = Vec::new();

    // Cut line into parts if they are too long for the terminal window
    let mut idx = 1;
    for line in entire_file.lines() {
        // If the line is longer than terminal window, save it's chunks separately
        if line.chars().count() > terminal_width {
            let subs = line.as_bytes().chunks(terminal_width).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();
            for s in subs {
                line_parts.push(s.to_string());
            }
        }
        // Otherwise, just throw entire line to line_parts
        else {
            line_parts.push(line.to_string());
        }

        idx+=1;
    }

    lines.push(line_parts.clone());

    // Summarize all that we know
    let about_this_line = Line {
        line_number: idx,
        parts_count: line_parts.len(),
        line_parts: lines.clone(),
    };
    // Finally, add all the required metadata to the "lines_data" list
    lines_data.push(about_this_line);

    editor(lines_data);
}

fn editor(lines_data:Vec<Line>) {
    // Collect some terminal info
    let terminal_width = terminal::size().unwrap().0 as usize;
    let terminal_height = terminal::size().unwrap().1 as usize;

    terminal::enable_raw_mode().unwrap();
    execute!(io::stdout(), terminal::EnterAlternateScreen).unwrap();
    execute!(io::stdout(), cursor::MoveTo(0,0)).unwrap();
    execute!(io::stdout(), terminal::SetTitle("Nota")).unwrap();

    for data in lines_data {
        for line in data.line_parts {
            for part in line {
                println!("{part}\r");
            }
        }
    }

    execute!(io::stdout(), cursor::MoveTo(0,0)).unwrap();

    // Cursor position
    let mut curx = 0;
    let mut cury = 0;

    // Actions on keyboard keypresses
    loop {
        let (action, _letter) = input::detect();
        match action {
            "STOP" => break,
            "UP" => {
                if cury > 0 {
                    cury-=1;
                }
            },
            "DOWN" => {
                if cury < terminal_height-2 {
                    cury+=1;
                }
            },
            "LEFT" => {
                if curx > 0 {
                    curx+=1;
                }
            },
            "RIGHT" => {
                if curx < terminal_width-1 {
                    curx+=1;
                }
            },
            _ => (),
        }
        execute!(io::stdout(), cursor::MoveTo(curx as u16, cury as u16)).unwrap();
    }

    terminal::disable_raw_mode().unwrap();
    execute!(io::stdout(), terminal::LeaveAlternateScreen).unwrap();
}
