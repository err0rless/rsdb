use colored::*;
use std::env;

use rustyline::error::ReadlineError;
use rustyline::Editor;

#[macro_use]
mod rsdb;

fn welcome_msg() {
    println!("rsdb: Linux debugger written in Rust");
    println!("  github: https://github.com/err0rless/rsdb");
    println!("  Type 'help' or '?' for help");
}

fn main() -> Result<(), i32> {
    match env::consts::OS {
        "linux" | "android" => welcome_msg(),
        _ => println!("rsdb only supports linux-based operating systems"),
    }

    // This holds target process ID, -1 if no process is attached
    let mut proc = rsdb::process::Proc::new();

    let mut reader = Editor::<()>::new();
    let shell = String::from("rsdb ~> ".bright_blue().to_string());
    loop {
        match reader.readline(shell.as_str()) {
            Ok(buffer) => {
                match rsdb::commandline::rsdb_main(&mut proc, &buffer) {
                    rsdb::commandline::MainLoopAction::Break => break,
                    rsdb::commandline::MainLoopAction::Continue => continue,
                    _ => (),
                }
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("rsdb interrupted, terminating...");
                break
            },
            Err(err) => {
                println!("Failed to read commandline {:?}", err);
                break
            }
        }
    }
    Ok(())
}
