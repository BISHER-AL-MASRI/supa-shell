/*
    This is a simple shell that uses termion for the terminal and shlex for parsing the input.
    It also uses the home crate to get the home directory of the user.

    I made this because I wanted to learn more about Rust and I wanted to make a simple shell.
    I also wanted to learn more about termion.
*/

use shlex::Shlex; // <- this is the crate that we will use to parse the input
use std::env; // <- this is the crate that we will use to get the environment variables
use std::fs; // <- this is the crate that we will use to read and write files
use std::fs::OpenOptions; // <- this is the crate that we will use to open files
use std::path::Path; // <- this is the crate that we will use to get the path of a file
use std::process::{Command, exit}; // <- this is the crate that we will use to execute commands (very weird to use)
use termion::event::Key; // <- this is the crate that we will use to listen for keyboard events
use termion::input::TermRead; // <- this is the crate that we will use to read from the terminal
use termion::raw::{IntoRawMode, RawTerminal}; // <- this is the crate that we will use to get the raw terminal
use std::io::{self, Write}; // <- this is the crate that we will use to write to the terminal
use termion::color; // <- this is the crate that we will use to change the color of the terminal

// This function checks if a command is a builtin command or not
fn is_builtin(command: &str) -> bool {
    matches!(
        command,
        "echo" | "exit" | "help" | "type" | "pwd" | "cd" | "history"
    )
}

// This function gets the completion candidates for a given input (for tab completion)
fn get_completion_candidates(input: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    // if the user typed nothing, return an empty vector
    if input.is_empty() {
        return candidates;
    }

    // if the user typed a slash, get the directory and prefix
    let (search_dir, prefix) = if input.contains('/') {
        let last_slash = input.rfind('/').unwrap();
        (&input[..=last_slash], &input[last_slash + 1..])
    } else {
        // if the user typed a dot, get the current directory and the input
        (".", input)
    };

    // if the directory exists, get the candidates
    if let Ok(entries) = fs::read_dir(search_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                // get the file name of the entry and convert it to a string
                let file_name = entry.file_name();
                let file_name_str = file_name.to_str().unwrap_or("");
                
                // if the file name starts with the prefix, add it to the candidates, prefix is the input without the slash
                if file_name_str.starts_with(prefix) {
                    let full_path = entry.path();
                    let candidate = if full_path.is_dir() {
                        format!("{}/", file_name_str)
                    } else {
                        file_name_str.to_string()
                    };
                    candidates.push(candidate);
                }
            }
        }
    }

    if candidates.is_empty() && search_dir == "." {
        if let Ok(paths) = env::var("PATH") {
            for path in paths.split(':') {
                if let Ok(dir_entries) = fs::read_dir(path) {
                    for entry in dir_entries {
                        if let Ok(entry) = entry {
                            let file_name = entry.file_name();
                            let file_name_str = file_name.to_str().unwrap_or("");
                            
                            if file_name_str.starts_with(prefix) && is_executable(&entry.path()) {
                                candidates.push(file_name_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    candidates.sort();
    candidates
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.is_file() && path.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn find_in_path(command: &str) -> Option<String> {
    if let Ok(paths) = env::var("PATH") {
        for path in paths.split(':') {
            let full_path = format!("{}/{}", path, command);
            if std::fs::metadata(&full_path).is_ok() {
                return Some(full_path);
            }
        }
    }
    None
}

fn reset_terminal(stdout: RawTerminal<io::Stdout>) {
    drop(stdout); 
}



fn main() {
    let stdout = io::stdout().into_raw_mode().unwrap();
    let raw_stdout = stdout;
    let mut stdout = raw_stdout.lock();

    // fix new line does \n\r
    stdout.write_all(b"\r\n").unwrap();

    let history_path = home::home_dir().unwrap().join(".shell_history");
    let mut history: Vec<String> = if let Ok(file) = std::fs::read_to_string(&history_path) {
        file.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    let mut history_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&history_path)
        .unwrap();

    let mut current_completion_candidates: Vec<String> = Vec::new();
    let mut current_completion_index = 0;

    // Hide the cursor (i like it like this) comment out if you want to see it
    let _ = write!(stdout, "{}",
        termion::cursor::Hide
    );

    loop {
        let mut input_string = String::new();

        write!(
            stdout,
            "\r{}{}{} ${} ",
            termion::color::Fg(color::LightGreen),
            termion::clear::CurrentLine,
            env::current_dir().unwrap().display(),
            termion::color::Fg(color::Reset)
        )
        .unwrap();
        stdout.flush().unwrap();
        stdout.flush().unwrap();
                    
        // Hide the cursor (i like it like this) comment out if you want to see it
        let _ = write!(stdout, "{}",
            termion::cursor::Hide
        );

        let stdin = io::stdin();
        for key in stdin.keys() {
            match key.unwrap() {
                Key::Char('\t') => {
                    if current_completion_candidates.is_empty() {
                        current_completion_candidates = get_completion_candidates(&input_string);
                        current_completion_index = 0;
                    }

                    if !current_completion_candidates.is_empty() {
                        write!(stdout, "\r$ {}", " ".repeat(input_string.len())).unwrap();
                        write!(stdout, "\r$ ").unwrap();

                        input_string = current_completion_candidates[current_completion_index].clone();
                        write!(stdout, "{}", input_string).unwrap();
                        
                        current_completion_index = 
                            (current_completion_index + 1) % current_completion_candidates.len();
                    }
                    
                    stdout.flush().unwrap();
                }
                Key::Char('\n') => {
                    current_completion_candidates.clear();
                    current_completion_index = 0;

                    write!(stdout, "\r\n").unwrap();
                    stdout.flush().unwrap();
                    break;
                }
                Key::Backspace => {
                    if !input_string.is_empty() {
                        input_string.pop();
                        write!(stdout, "\x08 \x08").unwrap();
                        stdout.flush().unwrap();
                    }
                }
                Key::Char(c) => {
                    current_completion_candidates.clear();
                    current_completion_index = 0;

                    input_string.push(c);
                    write!(stdout, "{}", c).unwrap();
                    stdout.flush().unwrap();
                }
                Key::Ctrl('c') => {
                    write!(stdout, "\r\nExiting...\r\n").unwrap();
                    reset_terminal(raw_stdout);
                    exit(0);
                }
                _ => {}
            }
        }
        write!(
            stdout,
            "{}{}",
            termion::style::Reset,
            termion::cursor::Show
        ).unwrap();
        stdout.flush().unwrap();
        
    let input = input_string.trim();
    if input.is_empty() {
        continue;
    }

    history.push(input_string.clone());
    history_file.write_all(input_string.as_bytes()).unwrap();
    history_file.write_all(b"\n").unwrap();

    let shlex = Shlex::new(&input_string);
    let parts: Vec<String> = shlex.collect();
    let command = parts[0].as_str();
    let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

    if is_builtin(command) {
        match command {
            "exit" => {
                write!(stdout, "\r\n").unwrap();
                reset_terminal(raw_stdout); 
                let exit_code = args.get(0).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                exit(exit_code);
            }
            "help" => {
                println!("Available commands: type, exit [code], help, echo, pwd, cd, history");
            }
            "echo" => {
                println!("{}", args.join(" "));
            }
            "pwd" => match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(err) => eprintln!("pwd: error retrieving current directory: {}", err),
            },
            "cd" => {
                if args.is_empty() {
                    eprintln!("cd: missing argument");
                    continue;
                }
                let path = args[0];
                if path == "~" {
                    let home_dir = env::var("HOME").expect("HOME environment variable not set");
                    if let Err(err) = env::set_current_dir(&home_dir) {
                        eprintln!("cd: {}: {}", home_dir, err);
                    }
                } else if let Err(err) = env::set_current_dir(path) {
                    eprintln!("cd: {}: {}", path, err);
                }
            }
            "history" => {
                for (i, cmd) in history.iter().enumerate() {
                    println!("{:>5}  {}", i + 1, cmd);
                }
            }
            _ => {}
        }
        continue;
    }

    if let Some(path) = find_in_path(command) {
        let output = Command::new(path).args(&args).output();
        match output {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    write!(stdout, "\r\n{}", String::from_utf8_lossy(&output.stdout)).unwrap();
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open("output.txt")
                        .unwrap();
                    file.write_all(output.stdout.as_slice()).unwrap();
                }
                if !output.stderr.is_empty() {
                    write!(stdout, "\r\n{}", String::from_utf8_lossy(&output.stderr)).unwrap();
                }
                stdout.flush().unwrap();
                                }
            Err(err) => {
                eprintln!("{}: failed to execute ({})", command, err);
            }
        }
    } else {
        eprintln!("{}: command not found", command);
    }
}
}