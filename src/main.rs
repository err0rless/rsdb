use colored::*;
use std::env;

use rustyline::error::ReadlineError;
use rustyline::Editor;

#[macro_use]
mod rsdb;

enum PlatformChecks {
    UnsupportedOS,
    UnsupportedArch,
}

fn platform_checks() -> Result<(), PlatformChecks> {
    match env::consts::ARCH {
        "x86_64" | "aarch64" => (),
        _ => return Err(PlatformChecks::UnsupportedArch),
    }
    match env::consts::OS {
        "linux" | "android" => (),
        _ => return Err(PlatformChecks::UnsupportedOS),
    }
    Ok(())
}

fn welcome_msg() {
    println!("rsdb: Linux debugger written in Rust");
    println!("  github: https://github.com/err0rless/rsdb");
    println!("  Arch  : {}", env::consts::ARCH);
    println!("  OS    : {}", env::consts::OS);
    println!("  Type 'help' or '?' for help");
}

fn main() -> Result<(), i32> {
    match platform_checks() {
        Err(err) => {
            println!("Unsupported platform: {}-{}", env::consts::ARCH, env::consts::OS);
            match err {
                PlatformChecks::UnsupportedArch => println!("  rsdb only supports: x86_64, AArch64"),
                PlatformChecks::UnsupportedOS =>   println!("  rsdb only supports: linux, android"),
            }
            return Err(1);
        },
        Ok(_) => welcome_msg(),
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
