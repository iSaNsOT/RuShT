use colored::*; // Import the colored crate
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Command, Stdio, Child};
use std::env;
use std::collections::HashMap;

fn main() {
    let mut current_dir = env::current_dir().unwrap_or_else(|_| Path::new("?").to_path_buf());
    let mut background_processes: HashMap<u32, Child> = HashMap::new();

    loop {
        let current_dir_str = current_dir.to_str().unwrap_or("?");
        print!(
            "{} {} ",
            current_dir_str.cyan().bold(), // Display the current path in cyan and bold
            ">".green().bold() // Display the prompt symbol in green and bold
        );
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let mut commands = input.trim().split('|').peekable();
        let mut previous_stdout = None;

        while let Some(command) = commands.next() {
            let mut parts = command.trim().split_whitespace();
            let mut command = parts.next().unwrap();
            let mut args: Vec<&str> = parts.collect();

            let run_in_background = command.ends_with('&') || args.last().map_or(false, |&arg| arg == "&");
            if run_in_background {
                if command.ends_with('&') {
                    command = &command[..command.len() - 1];
                } else {
                    args.pop();
                }
            }

            match command {
                "cd" => {
                    let new_dir = args.get(0).map_or("/", |&x| x);
                    let root = Path::new(new_dir);
                    if let Err(e) = std::env::set_current_dir(root) {
                        eprintln!("{}", format!("{}", e).red()); // Display errors in red
                    } else {
                        current_dir = env::current_dir().unwrap_or_else(|_| Path::new("?").to_path_buf());
                    }
                    previous_stdout = None;
                },
                "exit" => return,
                "jobs" => {
                    for (pid, _) in &background_processes {
                        println!("{}", format!("Background process running with PID: {}", pid).yellow());
                    }
                },
                "kill" => {
                    if let Some(pid_str) = args.get(0) {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            if let Some(mut child) = background_processes.remove(&pid) {
                                if let Err(e) = child.kill() {
                                    eprintln!("{}", format!("Failed to kill process {}: {}", pid, e).red());
                                } else {
                                    println!("{}", format!("Process {} terminated", pid).green());
                                }
                            } else {
                                eprintln!("{}", format!("No background process found with PID: {}", pid).red());
                            }
                        } else {
                            eprintln!("{}", format!("Invalid PID: {}", pid_str).red());
                        }
                    } else {
                        eprintln!("{}", "Usage: kill <PID>".red());
                    }
                },
                command => {
                    let stdin = previous_stdout
                        .take()
                        .map_or(Stdio::inherit(), |stdout| Stdio::from(stdout));

                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(&args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(mut child) => {
                            if run_in_background {
                                let pid = child.id();
                                println!("{}", format!("Process running in background with PID: {}", pid).green());
                                background_processes.insert(pid, child);
                            } else {
                                child.wait().unwrap();
                                previous_stdout = child.stdout.take();
                            }
                        }
                        Err(e) => {
                            previous_stdout = None;
                            eprintln!("{}", format!("Error: {}", e).red());
                        }
                    };
                }
            }
        }
    }
}