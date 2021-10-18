use std::ptr;
use libc::*;
use std::mem;
use nix::errno::Errno;

const NULL: *mut i32 = ptr::null_mut();
type VoidPtr = *mut libc::c_void;

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
        "SIGINT"  => Ok(libc::SIGINT),
        "SIGTERM" => Ok(libc::SIGTERM),
        "SIGBUS"  => Ok(libc::SIGBUS),
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
    waitpid(target, NULL, WSTOPPED) as i64
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

pub unsafe fn getregs(target: i32) -> Result<user_regs_struct, nix::errno::Errno> {
    let mut data = mem::MaybeUninit::uninit();
    let ret = ptrace(PTRACE_GETREGS, target, NULL, 
                         data.as_mut_ptr() as *const _ as *const c_void);
    Errno::result(ret)?;
    Ok(data.assume_init())
}

pub fn dumpregs(regs: &user_regs_struct) {
    println!("Dump Registers");
    println!("  rax: {:16x}", regs.rax);
    println!("  rbx: {:16x}", regs.rbx);
    println!("  rcx: {:16x}", regs.rcx);
    println!("  rdx: {:16x}", regs.rdx);
    println!("  rdi: {:16x}", regs.rdi);
    println!("  rdx: {:16x}", regs.rdx);
    println!("  r8 : {:16x}", regs.r8);
    println!("  r9 : {:16x}", regs.r9);
    println!("  r10: {:16x}", regs.r10);
    println!("  r11: {:16x}", regs.r11);
    println!("  r12: {:16x}", regs.r12);
    println!("  r13: {:16x}", regs.r13);
    println!("  r14: {:16x}", regs.r14);
    println!("  r15: {:16x}", regs.r15);
    println!("  rsp: {:16x}", regs.rsp);
    println!("  rbp: {:16x}", regs.rbp);
    println!("  rip: {:16x}", regs.rip);
}