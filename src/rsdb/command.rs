use core::panic;

use colored::*;
use nix::sys::wait::WaitStatus;
use nix::sys::signal::Signal;
use super::{process, ptrace};

pub enum MainLoopAction {
    None,
    Break,
    Continue,
}

// return signal name if @signum is available, "UNDEFINED" otherwise.
fn get_strsig(signum: i32) -> &'static str {
    unsafe {
        let sigcstr = libc::strsignal(signum);
        let sigstr = std::ffi::CStr::from_ptr(sigcstr)
            .to_str()
            .unwrap_or("UNDEFINED");
        sigstr
    }
}

pub fn attach(proc: &mut process::Proc, newtarget: i32) -> MainLoopAction {
    match unsafe { ptrace::attach_wait(newtarget) } {
        Ok(_) => {
            println!("Successfully attached to pid: {}", newtarget);
            *proc = process::Proc::from(newtarget);
        },
        Err(_) => (),
    }
    MainLoopAction::None
}

pub fn detach(proc: &mut process::Proc) -> MainLoopAction {
    if unsafe { ptrace::detach(proc.target).is_ok() } {
        proc.release();
    }
    MainLoopAction::None
}

pub fn cont(proc: &mut process::Proc) -> MainLoopAction {
    unsafe { ptrace::cont(proc.target).unwrap_or(-1); }

    // catching signal from the process
    match nix::sys::wait::waitpid(proc.get_pid(), None) {
        Ok(WaitStatus::Exited(_, exit_status)) => {
            println!("\nProgram terminated with status: {}", exit_status);
            proc.release();
        },
        Ok(WaitStatus::Stopped(_, signum)) => {
            let sigstr = get_strsig(signum as i32);
            match signum {
                Signal::SIGTERM => {
                    unsafe { ptrace::sigkill(proc.target).unwrap(); }

                    println!("\nProgram terminated with signal {}, {}", signum, sigstr);
                    proc.release();
                },
                _ => {
                    println!("\nProgram Stopped with signal {}, {}", signum, sigstr);
                    proc.getreg("rip")
                        .map(|pc| { println!("Stopped at 0x{:x}", pc); })
                        .unwrap_or_default();
                },
            }
        },
        Ok(WaitStatus::Signaled(_, signum, _)) => {
            let sigstr = get_strsig(signum as i32);
            match signum {
                Signal::SIGKILL => {
                    println!("\nProgram received {}, {}, terminating...", signum, sigstr);
                    proc.release(); 
                },
                _ => println!("Signaled {}", signum),
            }
        },
        Ok(status) => println!("\nProgram received status: {:?}", status),
        Err(err) => println!("waitpid failed: {:?}", err),
    }
    MainLoopAction::None
}

pub fn run(proc: &mut process::Proc) -> MainLoopAction {
    match proc.spawn_file() {
        -1 => MainLoopAction::None,
        child_pid => {
            // Wait parent until it's ready
            nix::sys::wait::wait().unwrap();
            proc.target = child_pid;

            // Continuing execution of the child
            super::command::cont(proc)
        },
    }
}

pub fn vmmap(proc: &mut process::Proc) -> MainLoopAction {
    proc.update();
    proc.dump_maps();
    MainLoopAction::None
}

pub fn kill(proc: &mut process::Proc) -> MainLoopAction {
    if unsafe { ptrace::sigkill(proc.target).is_ok() } {
        println!("Process killed successfully");
        proc.release();
    }
    MainLoopAction::None
}

pub fn quit(proc: &mut process::Proc) -> MainLoopAction {
    if proc.available() {
        println!("terminating the process({})...", proc.target);
        if unsafe { ptrace::sigkill(proc.target).is_ok() } {
            println!("Process killed successfully");
            proc.release();
        }
    }
    MainLoopAction::Break
}

// Subcommands of info
pub mod info {
    use super::*;

    pub fn regs(proc: &mut process::Proc) -> MainLoopAction {
        proc.dump_regs();
        MainLoopAction::None
    }

    pub fn proc(proc: &mut process::Proc) -> MainLoopAction {
        proc.update();
        proc.dump();
        MainLoopAction::None
    }
    
}