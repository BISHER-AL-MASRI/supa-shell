# Supa Shell

Supa Shell is a simple shell implementation built in Rust. It supports basic shell functionality like command execution, autocompletion, and command history management. The shell is designed to be used interactively, and it runs under the `supa` command.

## Features

- **Command Autocompletion:** Press the Tab key to complete commands or filenames.
- **Command History:** The shell keeps a history of commands executed in the session.
- **Built-in Commands:**
  - `echo`: Prints the arguments passed to it.
  - `exit [code]`: Exits the shell with an optional exit code.
  - `help`: Lists available commands.
  - `pwd`: Prints the current working directory.
  - `cd [path]`: Changes the current directory.
  - `history`: Shows the command history.

## Installation

To build Supa Shell, you need to have Rust installed. Follow the steps below to set up the project:

1. Clone the repository:
   ```sh
   git clone https://github.com/bisher-al-masri/supa-shell.git
   cd supa-shell
   ```

2. Build the project:
   ```sh
   cargo build --release
   ```

3. Run the shell:
   ```sh
   cargo run
   ```

## Usage

After running the shell, you'll see a prompt (`$ `). You can type commands, and the shell will process them. Here's an example of a session:

```sh
$ echo Hello, World!
Hello, World!

$ pwd
/home/user

$ cd /path/to/directory
$ history
   1  echo Hello, World!
   2  pwd
   3  cd /path/to/directory
```

Press `Tab` to autocomplete commands and file paths. Use `Ctrl + C` to exit the shell.

## Key Bindings

- **Tab:** Autocompletes the command or file path.
- **Backspace:** Deletes the last character.
- **Enter:** Executes the command.
- **Ctrl + C:** Exits the shell.

## File Structure

- **src/main.rs:** Main source file containing the implementation of the shell.
- **Cargo.toml:** Cargo configuration file for the Rust project.

## Contributing

Feel free to fork the repository and submit pull requests. Contributions are welcome!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.