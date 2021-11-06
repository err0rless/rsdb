use std::iter::*;
use std::mem;
use libc::WEXITSTATUS;
use regex::Regex;
use colored::*;


use super::{process, procfs, ptrace};

macro_rules! continue_if {
    ($cond:expr) => {
        if $cond {
            return MainLoopAction::Continue;
        }
    };
    ($cond:expr, $msg:tt) => {
        if $cond {
            println!("{}", $msg.red());
            return MainLoopAction::Continue;
        }
    };
}

pub enum MainLoopAction {
    None,
    Break,
    Continue,
}

fn rsdb_help() {
    println!("{}", "rsdb: Linux Debugger written in Rust".bright_yellow());
    println!("  help | ? => Print help");
    println!("  attach [PID | Package name] => attach to the prcess");
    println!("    e.g) {} or {}", "attach 31337".bright_yellow(), "attach com.test.package".bright_yellow());
    println!("  detach => detach from the process");
    println!("  run | r => run the process only if --file argument given");
    println!("             spawn the file and immediately stop it");
    println!("  info => info [Subcommand]");
    println!("    regs => show registers");
    println!("    proc => show process informations");
    println!("  vmmap | maps => show memory maps of the process");
    println!("  kill => send signal to the attached process");
    println!("  exit | quit => Exit rsdb");
}

pub fn rsdb_main(proc: &mut process::Proc, buffer: &String) -> MainLoopAction {
    let re = Regex::new(r"\s+").unwrap();
    let fullcmd = re.replace_all(buffer.trim(), " ");
    let commands = Vec::from_iter(fullcmd.split(" ").map(String::from));
    let command = &commands[0];
    
    let get_strsig = |signum: i32| -> &str {
        unsafe {
            let sigcstr = libc::strsignal(signum);
            let sigstr = std::ffi::CStr::from_ptr(sigcstr)
                .to_str()
                .unwrap_or("UNDEFINED");
            sigstr
        }
    };

    match command.as_str() {
        "attach" => {
            continue_if!(commands.len() != 2, "Usage: attach [PID | Package/Process name]");
            continue_if!(proc.available(), "rsdb is already holding the process, detach first");
            
            let process = &commands[1];
            let new_target = match process.parse::<i32>() {
                Ok(pid) => pid,
                Err(_) => procfs::findpid(process),
            };
            continue_if!(unsafe { !procfs::check_pid(new_target) }, 
                         "pid doesn't exist, check again");

            match unsafe { ptrace::attach_wait(new_target) } {
                Ok(_) => {
                    println!("Successfully attached to pid: {}", new_target);
                    proc.from(new_target);
                },
                Err(_) => (),
            }
        },
        "detach" => {
            continue_if!(!proc.available(), "No process has been attached");
            if unsafe { ptrace::detach(proc.target).is_ok() } {
                proc.clear();
            }
        },
        "continue" | "c" => {
            continue_if!(!proc.available(), "No process has been attached");
            unsafe {
                let _ = ptrace::cont(proc.target);
                let mut status = mem::MaybeUninit::<libc::c_int>::uninit();
                libc::waitpid(proc.target, status.as_mut_ptr() as *const _ as *mut libc::c_int, 0);

                // catching signal from the process
                match status.assume_init() {
                    s if libc::WIFEXITED(s) => {
                        proc.clear();
                        println!("\nProgram terminated with status: {}", WEXITSTATUS(s));
                    },
                    s if libc::WIFSTOPPED(s) => {
                        let stopsig = libc::WSTOPSIG(s);
                        let sigstr = get_strsig(stopsig);

                        match stopsig {
                            libc::SIGTERM => {
                                ptrace::sigkill(proc.target).unwrap();
                                proc.clear();

                                println!("\nProgram terminated with signal {}, {}", stopsig, sigstr);
                            },
                            _ => println!("\nProgram stopped with signal {}, {}", stopsig, sigstr),
                        }
                    },
                    s if libc::WIFSIGNALED(s) => {
                        match s {
                            libc::SIGKILL => {
                                println!("\nProgram killed from signal");
                                proc.clear();
                            },
                            _ => println!("Program received signal {}", get_strsig(s)),
                        }
                    },
                    s => println!("\nProgram received status {}", s),
                }
            };
        },
        "run" | "r" => {
            continue_if!(proc.available(), "rsdb is already holding the process, detach first");
            continue_if!(!proc.file_available(), "File is not available!");

            proc.spawn_file();
        },
        "info" => {
            continue_if!(commands.len() != 2, "Usage: info [Subcommand], help for more details");
            match commands[1].as_str() {
                "regs" | "r" => {
                    continue_if!(!proc.available(), "No process has been attached");
                    unsafe {
                        let regs = ptrace::getregs(proc.target);
                        continue_if!(regs.is_err(), "ptrace: Failed to retrive registers!");
    
                        let regs = regs.unwrap();
                        ptrace::dumpregs(&regs);
                    }
                },
                "proc" => {
                    continue_if!(!proc.available(), "No process has been attached");
                    proc.update();
                    proc.dump();
                },
                subcommand => println!("{}'{}'", "info: invalid subcommand: ".red(), subcommand),
            }
        },
        "vmmap" | "maps" => {
            continue_if!(!proc.available(), "No process has been attached");
            
            proc.update();
            proc.dump_maps();
        },
        "kill" => {
            continue_if!(commands.len() != 1, "Usage: kill");
            continue_if!(!proc.available(), "No process has been attached");

            if unsafe { ptrace::sigkill(proc.target).is_ok() } {
                println!("Process killed successfully");
                proc.clear();
            }
        },
        "exit" | "quit" | "q" => {
            if proc.available() {
                println!("terminating the process({})...", proc.target);
                if unsafe { ptrace::sigkill(proc.target).is_ok() } {
                    println!("Process killed successfully");
                    proc.clear();
                }
            }
            return MainLoopAction::Break;
        },
        "help" | "?" => rsdb_help(),
        "" => (),
        invalid_cmd => println!("{}: {}", "Invalid command".red(), invalid_cmd),
    }
    MainLoopAction::None
}
