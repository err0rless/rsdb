use colored::*;
use std::env;
use rustyline::error::ReadlineError;
use std::str::FromStr;
use clap::{App, Arg, ArgMatches};

#[macro_use]
mod rsdb;

enum PlatformChecks {
    UnsupportedOS,
    UnsupportedArch,
}

fn preprocess_arg_parser(proc: &mut rsdb::process::Proc, parser: &ArgMatches) {
    // -p, --pid <PID> 
    let arg_pid = parser.value_of("pid").unwrap_or("-1");
    proc.target = match i32::from_str(arg_pid).unwrap_or(-1) {
        -1 => -1,
        pid => unsafe { rsdb::ptrace::attach_wait(pid) }.unwrap_or(-1) as i32,
    }
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
    println!("rsdb, Linux debugger written in Rust");
    println!("  github: https://github.com/err0rless/rsdb");
    println!("  Arch  : {}", env::consts::ARCH);
    println!("  OS    : {}", env::consts::OS);
    println!("-> Type 'help' or '?' for help");
}

fn main() -> Result<(), i32> {
    // Commandline argument parser
    let arg_parser: ArgMatches = 
        App::new("rsdb: Linux debugger written in Rust")
            .version("0.0.0")
            .author("err0rless <err0rless313@gmail.com>")
            .arg(Arg::from_usage("-p, --pid <PID> 'Attach to a Specific Process ID'"))
            .get_matches();

    match platform_checks() {
        Err(err) => {
            println!("Unsupported platform: {}-{}", env::consts::ARCH, env::consts::OS);
            match err {
                PlatformChecks::UnsupportedArch => 
                    println!("  rsdb only supports: x86_64, AArch64"),
                PlatformChecks::UnsupportedOS =>   
                    println!("  rsdb only supports: linux, android"),
            }
            return Err(1);
        },
        Ok(_) => welcome_msg(),
    }

    // Singleton process object, it holds only one process.
    let mut proc = rsdb::process::Proc::new();
    preprocess_arg_parser(&mut proc, &arg_parser);

    // Commandline prerequisites for rustyline
    let mut reader = rustyline::Editor::<()>::new();
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
