use std::env;
use std::path::PathBuf;
use std::str::FromStr;

// Third-parties
use colored::*;
use rustyline::error::ReadlineError;
use clap::{App, Arg, ArgMatches};

mod rsdb;
mod cli;

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
    };

    // -f, --file <PATH>
    proc.file = match parser.value_of("file").unwrap_or("") {
        "" => PathBuf::from(""),
        file => {
            let filebuf = PathBuf::from(file);
            match filebuf.exists() && filebuf.is_file() {
                true => {
                    println!("Path to file is available: '{}'", file);
                    println!("  try 'run' to spawn the program");
                    filebuf
                },
                false => {
                    println!("Path to file is NOT available: '{}'", file);
                    PathBuf::from("")
                }
            }
        },
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
    println!("  Repo: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("  Arch: {}", env::consts::ARCH);
    println!("  OS  : {}", env::consts::OS);
    println!("-> Type 'help' or '?' for help");
}

fn enter_cli(proc: &mut rsdb::process::Proc) {
    use cli::command::MainLoopAction;

    // Commandline prerequisites for rustyline
    let mut reader = rustyline::Editor::<()>::new();
    let shell = String::from("rsdb ~> ".bright_blue().to_string());

    // Main commandline loop
    loop {
        match reader.readline(shell.as_str()) {
            Ok(buffer) => {
                match cli::rsdb_main(proc, &buffer) {
                    MainLoopAction::Break => break,
                    MainLoopAction::Continue => continue,
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
}

fn main() -> Result<(), i32> {
    // Commandline argument parser
    let arg_parser: ArgMatches = 
        App::new(env!("CARGO_PKG_DESCRIPTION"))
            .author(env!("CARGO_PKG_AUTHORS"))    
            .version(env!("CARGO_PKG_VERSION"))
            .version_short("v")
                .arg(
                    Arg::from_usage("-p, --pid <PID> 'Attach to a Specific Process ID'")
                        .required(false)
                        .conflicts_with("file")
                )
                .arg(
                    Arg::from_usage(concat!("-f, --file <PATH> 'Spawn a specific executable, ", 
                                            "empty string will be ignored'"))
                        .required(false)
                        .conflicts_with("pid")
                )
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
    enter_cli(&mut proc);
    Ok(())
}
