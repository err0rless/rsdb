use std::{io, fs};
use std::io::Write;
use std::iter::*;

use regex::Regex;

use colored::*;

// Unix/Linux
use nix::unistd::*;

// mods
#[macro_use]
mod rsdb;

// Continue if $cond is true
macro_rules! continue_if {
    ($cond:expr, $msg:tt) => {
        if $cond {
            println!("{}", $msg);
            continue
        }
    }
}

fn prelaunch_checks() -> Result<(), &'static str> {
    // Are you running rsdb on the OS where '/proc' file-system exists?
    match fs::File::open("/proc/self/maps") {
        Ok(_) => (),
        Err(_err) => return Err("rsdb failed to open '/proc/self/maps'"),
    }
    // Are you root?
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

    // This holds target process ID
    let mut target: i32 = -1;

    loop {
        buffer.clear();
        print!("{}", "rsdb # ".bright_blue());
        io::stdout().flush().unwrap();

        stdin.read_line(&mut buffer).unwrap();

        // trimming
        let fullcmd = re.replace_all(buffer.trim(), " ");
        let commands = Vec::from_iter(fullcmd.split(" ").map(String::from));
        let command = &commands[0];
        
        match command.as_str() {
            /*
             * Process attaching
             */
            "attach" => {
                continue_if!(commands.len() != 2, "Usage: attach {{PID | Package/Process name}}");
                continue_if!(target != -1, "rsdb is already holding the process, detach first");
                
                let process = &commands[1];
                target = match process.parse::<i32>() {
                    Ok(pid) => {
                        unsafe {
                            continue_if!(rsdb::process::check_pid(pid) != rsdb::process::KILL_SUCCESS, 
                                         "pid {} doesn't exist, check again");
                        }
                        pid
                    },
                    Err(_) => rsdb::process::findpid(process)
                };

                if target == -1 || !ptrace_check!("PTRACE_ATTACH", rsdb::ptrace::attach_wait(target)) {
                    target = -1;
                }
            },
            /*
             * Process detaching
             */
            "detach" => {
                continue_if!(target == -1, "error: No process has been attached");
                if ptrace_check!("PTRACE_DETACH", rsdb::ptrace::detach(target)) {
                    target = -1;
                }
            },
            "continue" | "c" => {
                continue_if!(target == -1, "error: No process has been attached");
                ptrace_check!("PTRACE_CONT", rsdb::ptrace::cont(target));
            },
            /*
             * Sending signal with PTRACE_KILL
             */
            "kill" => {
                continue_if!(commands.len() != 2, "Usage: kill {{KILL_SIGNAL}}");
                continue_if!(target == -1, "error: No process has been attached");
                
                let arg_signal = &commands[1];
                let signal_num = match arg_signal.parse::<i32>() {
                    Ok(signum) => signum,
                    Err(_) => {
                        let r = rsdb::ptrace::get_signum(arg_signal);
                        continue_if!(r.is_err(), "Invalid signal format!");
                        r.unwrap()
                    },
                };
                ptrace_check!("PTRACE_KILL", rsdb::ptrace::kill(target, signal_num));
            },
            /*
             * Quit rsdb, automatically detach the process if still attached
             */
            "exit" | "quit" | "q" => {
                if target != -1 {
                    ptrace_check!("auto PTRACE_DETACH", rsdb::ptrace::detach(target));
                }
                break; 
            },
            "help" | "?" => rsdb_help(), 
            "" => (),
            _ => println!("{}: {}", "Invalid command".red(), command),
        }
    }
    Ok(())
}