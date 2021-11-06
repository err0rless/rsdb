use std::mem;
use colored::*;
use super::{process, ptrace};

macro_rules! continue_if {
    ($cond:expr) => {
        if $cond {
            return command::MainLoopAction::Continue;
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
            proc.from(newtarget);
        },
        Err(_) => (),
    }
    MainLoopAction::None
}

pub fn detach(proc: &mut process::Proc) -> MainLoopAction {
    if unsafe { ptrace::detach(proc.target).is_ok() } {
        proc.clear();
    }
    MainLoopAction::None
}

pub fn cont(proc: &mut process::Proc) -> MainLoopAction {
    unsafe {
        let _ = ptrace::cont(proc.target);
        let mut status = mem::MaybeUninit::<libc::c_int>::uninit();
        libc::waitpid(proc.target, status.as_mut_ptr() as *const _ as *mut libc::c_int, 0);

        // catching signal from the process
        match status.assume_init() {
            s if libc::WIFEXITED(s) => {
                proc.clear();
                println!("\nProgram terminated with status: {}", libc::WEXITSTATUS(s));
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
    }
    MainLoopAction::None
}

pub fn run(proc: &mut process::Proc) -> MainLoopAction {
    proc.spawn_file();
    MainLoopAction::None
}

pub fn vmmap(proc: &mut process::Proc) -> MainLoopAction {
    proc.update();
    proc.dump_maps();
    MainLoopAction::None
}

pub fn kill(proc: &mut process::Proc) -> MainLoopAction {
    if unsafe { ptrace::sigkill(proc.target).is_ok() } {
        println!("Process killed successfully");
        proc.clear();
    }
    MainLoopAction::None
}

pub fn quit(proc: &mut process::Proc) -> MainLoopAction {
    if proc.available() {
        println!("terminating the process({})...", proc.target);
        if unsafe { ptrace::sigkill(proc.target).is_ok() } {
            println!("Process killed successfully");
            proc.clear();
        }
    }
    MainLoopAction::Break
}

// Subcommands of info
pub mod info {
    use super::*;

    pub fn regs(proc: &mut process::Proc) -> MainLoopAction {
        unsafe {
            let regs = ptrace::getregs(proc.target);
            if regs.is_err() {  }
            continue_if!(regs.is_err(), "ptrace: Failed to retrive registers!");

            let regs = regs.unwrap();
            ptrace::dumpregs(&regs);
        }
        MainLoopAction::None
    }

    pub fn proc(proc: &mut process::Proc) -> MainLoopAction {
        proc.update();
        proc.dump();
        MainLoopAction::None
    }
    
}