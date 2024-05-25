use std::io;
use std::io::Write;
use crossterm::terminal::{self, Clear, ClearType};
use carrot_libs::input;

fn flush() {
    io::stdout().flush().expect("Failed to flush terminal output!")
}

pub fn get(prompt:String,startingtext:Option<String>,startingposition:usize) -> String {
    // FOR ALL COMMENTS BELLOW: Assume, that user typed this command into a shell: af file then ad dir
    // This variable contains full line typed by the user (List 1.: 'af file then ad dir')
    let mut part:Vec<char> = if startingtext.is_some() {
        startingtext.clone().unwrap().chars().collect::<Vec<char>>().to_vec()
    } else {
        Vec::new()
    };

    // Print a prompt
    print!("{prompt}");
    // Print starting text if defined
    print!("{}\r", startingtext.unwrap_or("".to_string()));

    // Flush stdout to print the prompt
    io::stdout().flush().expect("Cannot flush output!");
        // Read line into "part"
        // Process each character written on keyboard

        // Get the cursor position when we've started
        let initial_cur_pos = crossterm::cursor::position().expect("Failed to obtain cursor position!").0;
        // This is going to indicate on which index we want to add new letters to "part"
        let mut idx = startingposition;

        loop {
            // Move to start of the column
            print!("\r");
            // Clear everything on that line
            print!("{}", Clear(ClearType::CurrentLine));
            // Show prompt and contents of input
            let part_to_string = String::from_iter(&part);
            print!("{}{}", prompt, part_to_string);
            // Move cursor to position defined in "idx" + "initial_cur_pos"
            print!("{}", crossterm::cursor::MoveToColumn(idx as u16 + initial_cur_pos)); 
            // Flush on start and end of the loop
            flush();

            // Collect some terminal info
            let terminal_width = terminal::size().unwrap().0 as usize;
            if part.len() > terminal_width {
                return String::from_iter(part);
            }

            let (key_type, letter) = input::detect();
            // Check event
            match key_type {
                // CTRL+Z or CTRL+D: Quit
                "STOP" => {
                    // Inspired by BASH
                    // Don't exit unless the prompt is empty
                    if part.is_empty() {
                        // Disable raw mode and quit
                        crossterm::terminal::disable_raw_mode().expect("Cannot quit from raw terminal mode!");
                        println!();
                        // process::exit(1);
                    }
                },

                // ARROWS without CTRL: Cursor movement
                "LEFT" => {
                    if idx != 0 {
                        // Move cursor to left
                        idx -= 1;
                    } else {print!("\x07");continue;};
                    
                },
                "RIGHT" => {
                    if idx != part.len() {
                        // Move cursor to right
                        idx += 1;
                    } else {print!("\x07");continue;};
                },

                // CTRL+ARROW: Move cursor to the next whitespace
                "CTRL+LEFT" => {
                    while idx != 0 {
                        idx -= 1;
                        if part[idx].is_whitespace() { break }
                    }
                }
                "CTRL+RIGHT" => {
                    while idx != part.len() {
                        idx += 1;
                        if idx == part.len() || part[idx].is_whitespace() { break }
                    }
                }
                
                // HOME and END keys support
                "HOME" => {
                    // Move cursor back to the prompt
                    idx=0;
                }
                "END" => {
                    // Move where "part" is reaching it's end
                    idx=part.len();
                }

                // BACKSPACE: Remove character before cursor
                "BACKSPACE" => {
                    if idx != 0 {
                        if idx != part.len() {
                            part.remove(idx-1);
                        }
                        else {
                            // Since removing from the last index is impossibl, use "pop" when user wants
                            // to remove the last character from the part
                            part.pop();
                        };
                        // Move cursor
                        idx -= 1;
                    } else {print!("\x07")};
                },
                // CTRL+BACKSPACE: Remove character before cursor until whitespace
                // FUNFACT: Terminal emulators on Linux detect CTRL+Backspace as CTRL+H
                // The code below is correct. Don't change KeyCode::Char to KeyCode::Spacebar
                "CTRL+BACKSPACE" => {
                    while idx > 0 {
                        if !part[idx-1].is_whitespace() {
                            part.remove(idx-1);
                        }
                        else {
                            // Remove the remaining white space
                            part.remove(idx-1);
                            idx-=1;
                            break;
                        }
                        idx-=1;
                    }
                },

                // DEL: Remove character on cursor
                "DEL" => {
                    if idx != part.len() {
                        part.remove(idx);
                    } else {print!("\x07")};
                },
                // CTRL+DEL: Remove all characters after cursor until whitespace
                "CTRL+DEL" => {
                    while idx < part.len() {
                        if !part[idx].is_whitespace() {
                            part.remove(idx);
                        }
                    }
                },

                // ENTER: Quickly append newline character to "part" and stop waiting for input by breaking out of the loop
                "ENTER" => {
                    part.push('\n');
                    crossterm::terminal::disable_raw_mode().unwrap();
                    return String::from_iter(part);
                },
                "ESCAPE" => {
                    crossterm::terminal::disable_raw_mode().unwrap();
                    return String::from_iter(part);
                },
               
                // OTHER
                "UNKNOWN" => {
                    // Bell!
                    print!("\x07");
                },

                // ANY CHARACTER WITHOUT CTRL: Show it on keyboard and add it to "part" variable
                "CHAR" => {
                    // Insert a char in "part" on position where the cursor is located + the number 
                    part.insert(idx, letter.expect(""));
                    // Move cursor to the right as we type
                    idx +=1;
                },
                _ => {
                    print!("\x07");
                    // process::exit(1);
                }
            };
        };
}
