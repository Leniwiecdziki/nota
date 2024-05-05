use carrot_libs::{args, input};
use crossterm::{terminal, cursor, execute};
use std::process;
use std::io::{self, Read};
use std::fs;
use std::str;
use std::collections::hash_map::HashMap;

// Type used for storing information about each file line
#[derive(Debug)]
struct Line {
    // It's number in file
    line_number: usize,
    // Number of parts that were created to fit in terminal
    parts_count: usize,
    // Parts!
    line_parts: Vec<HashMap<usize, String>>,
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

    // Cut line into parts if they are too long for the terminal window
    let mut line_index = 1;
    for line in entire_file.lines() {
        // This thing is a list which stores list of line parts with their ID
        let mut lines = Vec::new();

        // Just the parts extracted from large line
        let mut line_parts = HashMap::new();

        let mut part_index = 0;
        // If the line is longer than terminal window, save it's chunks separately
        if line.chars().count() > terminal_width {
            let subs = line.as_bytes().chunks(terminal_width).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();
            for s in subs {
                line_parts.insert(line_index+part_index, s.to_string());
                part_index+=1;
            }
        }
        // Otherwise, just throw entire line to line_parts
        else {
            line_parts.insert(line_index+part_index, line.to_string());
        }
        lines.push(line_parts.clone());

        // Summarize all that we know about the current line
        let about_this_line = Line {
            line_number: line_index,
            parts_count: line_parts.len(),
            line_parts: lines,
        };
        // Finally, add all the required metadata to the "lines_data" list
        lines_data.push(about_this_line);

        line_index+=1;
    }

    editor(lines_data);
}

fn editor(lines_data:Vec<Line>) {
    // Collect some terminal info
    let terminal_width = terminal::size().unwrap().0 as usize;
    let terminal_height = terminal::size().unwrap().1 as usize;

    dbg!(&lines_data);

    terminal::enable_raw_mode().unwrap();
    execute!(io::stdout(), terminal::EnterAlternateScreen).unwrap();
    execute!(io::stdout(), cursor::MoveTo(0,0)).unwrap();
    execute!(io::stdout(), terminal::SetTitle("Nota")).unwrap();

    let mut idx = 1;
    for line in &lines_data {
        for part in &line.line_parts {
            println!("{}\r", part.get(&idx).unwrap() );
            idx+=1;
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
