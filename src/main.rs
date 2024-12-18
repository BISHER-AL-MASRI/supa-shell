use shlex::Shlex;
use std::env;
use std::fs::OpenOptions;
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

// plan: tab completion: look thru /bin, /usr/bin, etc and find best match, if the user has typed nothing then do \t and not tab completion
// * Broken
#[allow(dead_code)]
fn tab_completion(input: &str) -> Option<String> {
    let mut paths = std::env::var("PATH").unwrap_or_default();
    paths.push_str(":/bin:/usr/bin");
    for path in paths.split(':') {
        let full_path = format!("{}/{}", path, input);
        if std::fs::metadata(&full_path).is_ok() {
            return Some(full_path);
        }
    }

    None
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

    loop {
        write!(stdout, "\r$ ").unwrap();
        stdout.flush().unwrap();

        let mut input_string = String::new();
        let stdin = io::stdin();
        for key in stdin.keys() {
            match key.unwrap() {
                Key::Char('\t') => {
                    // TODO: add tab completion
                    
                }
                Key::Char('\n') => {
                    write!(stdout, "\n").unwrap();
                    stdout.flush().unwrap();
                    break;
                }
                Key::Char(c) => {
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
