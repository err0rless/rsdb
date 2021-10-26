use std::{io, fs, path::PathBuf};
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
            println!("{}", $msg.red());
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
    println!("  help | ? => Print help");
    println!("  attach {{PID | Package name}} => attach to the prcess");
    println!("    e.g) {} or {}", "attach 31337".bright_yellow(), "attach com.test.package".bright_yellow());
    println!("  detach => detach from the process");
    println!("  info => info {{subcommand}}");
    println!("    regs => show registers");
    println!("    proc => show process informations");
    println!("    maps => show memory maps for the process");
    println!("  kill => send signal to the attached process");
    println!("  exit | quit => Exit rsdb");
}

fn main() -> Result<(), i32> {
    if let Err(err_code) = prelaunch_checks() {
        println!("failed to launch rsdb: {}", err_code.red());
        return Err(1);
    }

    let stdin = io::stdin();
    let mut buffer = String::new();

    let re = Regex::new(r"\s+").unwrap();

    // This holds target process ID, -1 if no process is attached
    let mut proc = rsdb::process::Proc {
        target: -1, 
        cmdline: String::from(""),
        exe: PathBuf::new(), 
        cwd: PathBuf::new() 
    };
    
    let commandline = String::from("rsdb ~> ".bright_blue().to_string());
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
                continue_if!(proc.target != -1, "rsdb is already holding the process, detach first");
                
                let process = &commands[1];
                let new_target = match process.parse::<i32>() {
                    Ok(pid) => pid,
                    Err(_) => rsdb::process::findpid(process),
                };
                continue_if!(unsafe { !rsdb::process::check_pid(new_target) }, 
                             "pid doesn't exist, check again");

                match unsafe { rsdb::ptrace::attach_wait(new_target) } {
                    Ok(_) => {
                        println!("Successfully attached to pid: {}", new_target);
                        proc.init_with_pid(new_target);
                    },
                    Err(_) => (),
                }
            },
            "detach" => {
                continue_if!(proc.target == -1, "No process has been attached");
                if unsafe { rsdb::ptrace::detach(proc.target).is_ok() } {
                    proc.clear();
                }
            },
            "continue" | "c" => {
                continue_if!(proc.target == -1, "No process has been attached");
                unsafe { let _ = rsdb::ptrace::cont(proc.target); };
            },
            "info" => {
                continue_if!(commands.len() != 2, "Usage: info {{regs | proc}}");
                let arg = &commands[1];
                match &arg[..] {
                    "regs" | "r" => {
                        continue_if!(proc.target == -1, "No process has been attached");
                        unsafe {
                            let regs = rsdb::ptrace::getregs(proc.target);
                            continue_if!(regs.is_err(), "ptrace: Failed to retrive registers!");
        
                            let regs = regs.unwrap();
                            rsdb::ptrace::dumpregs(&regs);
                        }
                    },
                    "proc" => {
                        continue_if!(proc.target == -1, "No process has been attached");
                        proc.dump();
                    },
                    "maps" => {
                        continue_if!(proc.target == -1, "No process has been attached");
                        
                        let maps = rsdb::process::get_proc_maps(proc.target);
                        continue_if!(maps.is_err(), "process: Failed to get memory maps");
                        
                        println!("{}", maps.unwrap());
                    },
                    _ => println!("{}'{}'", "info: invalid subcommand: ".red(), arg),
                }
            },
            "kill" => {
                continue_if!(commands.len() != 1, "Usage: kill");
                continue_if!(proc.target == -1, "No process has been attached");

                if unsafe { rsdb::ptrace::sigkill(proc.target).is_ok() } {
                    println!("Process killed successfully");
                    proc.clear();
                }
            },
            "exit" | "quit" | "q" => break,
            "help" | "?" => rsdb_help(),
            "" => (),
            _ => println!("{}: {}", "Invalid command".red(), command),
        }
    }
    Ok(())
}
