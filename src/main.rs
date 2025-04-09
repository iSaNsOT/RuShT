use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Command, Child, Stdio};
use std::env;

fn main() {
    // Get the initial current working directory
    let mut current_dir = env::current_dir().unwrap_or_else(|_| Path::new("?").to_path_buf());

    loop {
        // Print the prompt with the current directory
        let current_dir_str = current_dir.to_str().unwrap_or("?");
        print!("{} > ", current_dir_str);
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // Pipe implementation
        let mut commands = input.trim().split('|').peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = std::env::set_current_dir(root) {
                        eprintln!("{}", e);
                    } else {
                        // Update the current directory if the change was successful
                        current_dir = env::current_dir().unwrap_or_else(|_| Path::new("?").to_path_buf());
                    }
                    previous_command = None;
                },
                "exit" => return,
                command => {
                    let stdin = previous_command
                        .map_or(Stdio::inherit(), |output: Child| Stdio::from(output.stdout.unwrap()));

                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => {
                            previous_command = Some(output);
                        }
                        Err(e) => {
                            previous_command = None;
                            eprintln!("Error: {}", e);
                        }
                    };
                }
            }
        }
        if let Some(mut final_command) = previous_command {
            // Block until the last command finishes
            final_command.wait().unwrap();
        }
    }
}