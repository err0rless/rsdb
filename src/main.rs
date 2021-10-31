use std::fs;
use colored::*;
use nix::unistd::*;

use rustyline::error::ReadlineError;
use rustyline::Editor;

#[macro_use]
mod rsdb;

fn prelaunch_checks() -> Result<(), &'static str> {
    // rsdb needs '/proc' pseudo file system to run.
    match fs::File::open("/proc/self/maps") {
        Ok(_) => (),
        Err(_err) => return Err("rsdb failed to open '/proc/self/maps'"),
    }
    // @TODO: remove this constraint, let rsdb runs a process as a child of it.
    match Uid::effective().is_root() {
        true => (),
        false => return Err("Please run rsdb with root privilege"),
    }
    Ok(())
}

fn main() -> Result<(), i32> {
    if let Err(err_code) = prelaunch_checks() {
        println!("failed to launch rsdb: {}", err_code.red());
        return Err(1);
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
