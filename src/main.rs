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
    if let Some(arg_pid) = parser.value_of("pid") {
        if let Ok(pid) = i32::from_str(arg_pid) {
            if ptrace::attach_wait(pid).is_ok() {
                session.set_target(pid as i32).unwrap_or(0);

                // print current pc
                let pc = session.proc.getreg("pc").unwrap_or_default();
                println!("Successfully attached to pid: {}", pid);
                println!("Stopped at: pc={:#x}", pc);
                
                // set elf with '/proc/{PID}/exe'
                match session.set_elf(session.get_exe().to_path_buf()) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("[ELF] Failed to parse an ELF");
                        println!("  path: '{}'", session.get_exe().to_path_buf().display());
                        println!("  err : {:?}", e);
                    },
                }
            }
        }
    }

    // -f, --file <PATH>
    if let Some(file_str) = parser.value_of("file") {
        match session.set_elf(PathBuf::from(file_str)) {
            Ok(_) => {
                println!("Path to file is available: '{}'", file_str);
                println!("  try 'run' to spawn the program");
            },
            Err(e) => {
                println!("[ELF] Failed to parse an ELF");
                println!("  path: '{}'", file_str);
                println!("  err : {:?}", e);
            },
        }
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
