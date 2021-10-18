use std::{io, fs};
use std::io::Write;
use std::iter::*;
use regex::Regex;
use colored::*;
use nix::unistd::*;

#[macro_use]
mod rsdb;

macro_rules! continue_if {
    ($cond:expr) => {
        if $cond {
            continue
        }
    };
    ($cond:expr, $msg:tt) => {
        if $cond {
            println!("{}", $msg);
            continue
        }
    };
}

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

fn rsdb_help() {
    println!("{}", "rsdb: Linux Debugger written in Rust".bright_yellow());
    println!("    help | ? => Print help");
    println!("    attach {{PID | Package name}} => attach to the prcess");
    println!("        e.g) {} or {}", "attach 31337".bright_yellow(), "attach com.test.package".bright_yellow());
    println!("    detach => detach from the process");
    println!("    kill => send signal to the attached process");
    println!("    exit | quit => Exit rsdb");
}

fn main() -> Result<(), i32> {
    if let Err(err_code) = prelaunch_checks() {
        println!("rsdb: {}", err_code.red());
        return Err(1);
    }

    let stdin = io::stdin();
    let mut buffer = String::new();

    let re = Regex::new(r"\s+").unwrap();

    // This holds target process ID, -1 if no process is attached
    let mut target: i32 = -1;

    let mut commandline = String::from("rsdb # ".bright_blue().to_string());
    loop {
        buffer.clear();
        print!("{}", commandline);
        io::stdout().flush().unwrap();

        stdin.read_line(&mut buffer).unwrap();

        let fullcmd = re.replace_all(buffer.trim(), " ");
        let commands = Vec::from_iter(fullcmd.split(" ").map(String::from));
        let command = &commands[0];
        
        match command.as_str() {
            "attach" => {
                continue_if!(commands.len() != 2, "Usage: attach {{PID | Package/Process name}}");
                continue_if!(target != -1, "rsdb is already holding the process, detach first");
                
                let process = &commands[1];
                target = match process.parse::<i32>() {
                    Ok(pid) => {
                        unsafe {
                            continue_if!(rsdb::process::check_pid(pid) != rsdb::process::KILL_SUCCESS, 
                                         "pid doesn't exist, check again");
                        }
                        pid
                    },
                    Err(_) => rsdb::process::findpid(process)
                };

                // one of attaching and waiting pid failed, nullify target pid
                if unsafe { rsdb::ptrace::attach_wait(target).is_err() } {
                    target = -1;
                }
            },
            "detach" => {
                continue_if!(target == -1, "error: No process has been attached");
                if unsafe { rsdb::ptrace::detach(target).is_ok() } {
                    target = -1;
                    commandline = String::from("rsdb # ".bright_blue().to_string());
                }
            },
            "continue" | "c" => {
                continue_if!(target == -1, "error: No process has been attached");
                unsafe { let _ = rsdb::ptrace::cont(target); };
            },
            "regs" => {
                continue_if!(target == -1, "error: No process has been attached");
                unsafe {
                    let regs = rsdb::ptrace::getregs(target);
                    continue_if!(regs.is_err(), "Failed to retrive registers!");

                    let regs = regs.unwrap();
                    rsdb::ptrace::dumpregs(&regs);
                }
            },
            "kill" => {
                continue_if!(commands.len() != 2, "Usage: kill {{KILL_SIGNAL}}");
                continue_if!(target == -1, "error: No process has been attached");
                
                let arg_signal = &commands[1];
                let signum = match arg_signal.parse::<i32>() {
                    Ok(_signum) => _signum,
                    Err(_) => {
                        let r = rsdb::ptrace::get_signum(arg_signal);
                        continue_if!(r.is_err(), "Invalid signal format!");
                        r.unwrap()
                    },
                };
                unsafe { let _ = rsdb::ptrace::kill(target, signum); };
            },
            "exit" | "quit" | "q" => break,
            "help" | "?" => rsdb_help(),
            "" => (),
            _ => println!("{}: {}", "Invalid command".red(), command),
        }
    }
    Ok(())
}
