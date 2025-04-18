#![warn(clippy::all, clippy::pedantic)]
use colored::*;
use rustyline::{Editor, Config};
use rustyline::error::ReadlineError;
use std::path::Path;
use std::process::{Command, Stdio, Child};
use std::env;
use std::collections::HashMap;

fn main() {
    let mut current_dir = env::current_dir().unwrap_or_else(|_| Path::new("?").to_path_buf());
    let mut background_processes: HashMap<u32, Child> = HashMap::new();

    // Create a rustyline Editor with default configuration
    let config = Config::builder().build();
    let mut rl = Editor::<(), rustyline::history::MemHistory>::with_history(config, rustyline::history::MemHistory::default()).expect("Failed to create Editor");

    loop {
        let current_dir_str = current_dir.to_str().unwrap_or("?");
        let prompt = format!(
            "{} {} ",
            current_dir_str.cyan().bold(),
            ">".green().bold()
        );

        // Read input using rustyline
        let input = match rl.readline(&prompt) {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                line
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "CTRL+C pressed. Exiting shell...".yellow());
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "CTRL+D pressed. Exiting shell...".yellow());
                break;
            }
            Err(err) => {
                eprintln!("{}", format!("Error reading line: {}", err).red());
                break;
            }
        };

        let mut commands = input.trim().split('|').peekable();
        let mut previous_stdout = None;

        while let Some(command) = commands.next() {
            let mut parts = command.split_whitespace();
            let mut command = parts.next().unwrap();
            let mut args: Vec<&str> = parts.collect();

            let run_in_background = command.ends_with('&') || args.last().is_some_and(|&arg| arg == "&");
            if run_in_background {
                if command.ends_with('&') {
                    command = &command[..command.len() - 1];
                } else {
                    args.pop();
                }
            }

            match command {
                "cd" => {
                    let new_dir = args.first().map_or("/", |&x| x);
                    let root = Path::new(new_dir);
                    if let Err(e) = std::env::set_current_dir(root) {
                        eprintln!("{}", format!("{}", e).red());
                    } else {
                        current_dir = env::current_dir().unwrap_or_else(|_| Path::new("?").to_path_buf());
                    }
                    previous_stdout = None;
                },
                "exit" => return,
                "jobs" => {
                    for pid in background_processes.keys() {
                        println!("{}", format!("Background process running with PID: {}", pid).yellow());
                    }
                },
                "kill" => {
                    if let Some(pid_str) = args.first() {
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
                        .map_or(Stdio::inherit(), Stdio::from);

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