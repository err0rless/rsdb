use std::env;
use std::path::PathBuf;
use std::str::FromStr;

// Third-parties
use colored::*;
use rustyline::error::ReadlineError;
use clap::{App, Arg, ArgMatches};

mod traits;

mod session;
mod cli;
mod process;
mod ptrace;

enum PlatformChecks {
    UnsupportedOS,
    UnsupportedArch,
}

fn preprocess_arg_parser(session: &mut session::Session, parser: &ArgMatches) {
    // -p, --pid <PID> 
    let arg_pid = parser.value_of("pid").unwrap_or("-1");
    let target = match i32::from_str(arg_pid).unwrap_or(-1) {
        -1 => -1,
        pid => unsafe { ptrace::attach_wait(pid) }.unwrap_or(-1) as i32,
    };
    session.set_target(target).unwrap_or(-1);

    // -f, --file <PATH>
    match parser.value_of("file").unwrap_or("") {
        "" => (),
        file => {
            let filebuf = PathBuf::from(file);
            if filebuf.exists() && filebuf.is_file() {
                println!("Path to file is available: '{}'", file);
                println!("  try 'run' to spawn the program");

                match session.set_elf(filebuf) {
                    Ok(_) => (),
                    Err(e) => println!("[ELF] Error: {:?}", e),
                };
            }
            else {
                println!("Path to file is NOT available: '{}'", file);
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

fn enter_cli(session: &mut session::Session) {
    use cli::command::MainLoopAction;

    // Commandline prerequisites for rustyline
    let mut reader = rustyline::Editor::<()>::new();
    let shell = String::from("rsdb ~> ".bright_blue().to_string());

    // Main commandline loop
    loop {
        match reader.readline(shell.as_str()) {
            Ok(buffer) => {
                match cli::rsdb_main(session, &buffer) {
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
    let mut session = session::Session::new();

    preprocess_arg_parser(&mut session, &arg_parser);
    enter_cli(&mut session);
    Ok(())
}
