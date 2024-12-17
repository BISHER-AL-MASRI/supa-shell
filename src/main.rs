use shlex::Shlex;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::{self, Command};

fn is_builtin(command: &str) -> bool {
    matches!(command, "echo" | "exit" | "help" | "type" | "pwd" | "cd" | "history")
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

fn main() {
    let stdin = io::stdin();
    let history_path = home::home_dir().unwrap().join(".shell_history");

    // Load history into a vector
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
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let input = if input.starts_with('!') {
            if let Ok(index) = input[1..].parse::<usize>() {
                if let Some(command) = history.get(index - 1) {
                    println!("{}", command);
                    command.clone()
                } else {
                    eprintln!("history: {}: event not found", index);
                    continue;
                }
            } else {
                eprintln!("history: invalid event");
                continue;
            }
        } else {
            input.to_string()
        };

        // Add command to history
        history.push(input.clone());
        history_file.write_all(input.as_bytes()).unwrap();
        history_file.write_all(b"\n").unwrap();

        let shlex = Shlex::new(&input);
        let parts: Vec<String> = shlex.collect();
        let command = parts[0].as_str();
        let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

        // Builtin commands
        if is_builtin(command) {
            match command {
                "exit" => {
                    let exit_code = args.get(0).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                    process::exit(exit_code);
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

        // External commands
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
                Err(_) => {
                    eprintln!("{}: failed to execute", command);
                }
            }
        } else {
            eprintln!("{}: command not found", command);
        }
    }
}
