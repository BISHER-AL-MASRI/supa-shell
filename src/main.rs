use shlex::Shlex;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::{self, Write};
use std::process::{Command, exit};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

fn is_builtin(command: &str) -> bool {
    matches!(
        command,
        "echo" | "exit" | "help" | "type" | "pwd" | "cd" | "history"
    )
}

fn get_completion_candidates(input: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    if input.is_empty() {
        return candidates;
    }

    let (search_dir, prefix) = if input.contains('/') {
        let last_slash = input.rfind('/').unwrap();
        (&input[..=last_slash], &input[last_slash + 1..])
    } else {
        (".", input)
    };

    if let Ok(entries) = fs::read_dir(search_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_str().unwrap_or("");
                
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
    
        loop {
            write!(stdout, "\r$ ").unwrap();
            stdout.flush().unwrap();
    
            let mut input_string = String::new();
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
    
                        write!(stdout, "\n").unwrap();
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
                        write!(stdout, "\nExiting...\n").unwrap();
                        reset_terminal(raw_stdout);
                        exit(0);
                    }
                    _ => {}
                }
            }
            
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
                    write!(stdout, "\n").unwrap();
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
                        print!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    if !output.stderr.is_empty() {
                        eprint!("{}", String::from_utf8_lossy(&output.stderr));
                    }
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
