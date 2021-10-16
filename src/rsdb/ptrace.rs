use std::ptr;
use libc::*;

use colored::*;

const NULL: *mut i32 = ptr::null_mut();

#[macro_export]
macro_rules! ptrace_check {
    ($ptrace_command: tt, $ptrace_expr: expr) => {
        unsafe {
            if $ptrace_expr != -1 {
                println!("Successfully done: {}", $ptrace_command);
                true
            }
            else {
                println!("Failed to finish: {}", $ptrace_command);
                false
            }
        }
    };
}

pub fn get_signum(signal_name: &str) -> Result<i32, ()> {
    let upper = signal_name.to_uppercase();
    match &upper[..] {
        "SIGKILL" => Ok(libc::SIGKILL),
        "SIGCONT" => Ok(libc::SIGCONT),
        "SIGABRT" => Ok(libc::SIGABRT),
        "SIGSTOP" => Ok(libc::SIGSTOP),
        "SIGINT" => Ok(libc::SIGINT),
        "SIGTERM" => Ok(libc::SIGTERM),
        "SIGBUG" => Ok(libc::SIGBUS),
        "SIGSEGV" => Ok(libc::SIGSEGV),
        "SIGTRAP" => Ok(libc::SIGTRAP),
        _ => Err(()),
    }
}

pub unsafe fn attach(target: i32) -> i64 {
    ptrace(PTRACE_ATTACH, target, NULL, NULL)
}

pub unsafe fn attach_wait(target: i32) -> i64 {
    if attach(target) == -1 { return -1 }
    waitpid(target, NULL, 0) as i64
}

pub unsafe fn detach(target: i32) -> i64 {
    ptrace(PTRACE_DETACH, target, NULL, NULL)
}

pub unsafe fn cont(target: i32) -> i64 {
    ptrace(PTRACE_CONT, target, NULL, NULL)
}

pub unsafe fn kill(target: i32, signal: i32) -> i64 {
    ptrace(PTRACE_KILL, target, signal, NULL)
}