use std::ptr;
use libc::*;
use std::mem;
use nix::errno::Errno;

use colored::*;

const NULL: *mut i32 = ptr::null_mut();

// print error if global error is set, similar to perror(...);
#[macro_export]
macro_rules! rsdb_ptrace {
    ($($ptrace_args: expr),*) => {
        /* unsafe */ {
            let ret = libc::ptrace($($ptrace_args), *);
            if let Err(no) = Errno::result(ret) {
                let errstr: String = format!("ptrace: {}", no.desc());
                println!("{}", errstr.red());
                Err(())
            }
            else {
                Ok(ret)
            }
        }
    };
}

pub unsafe fn attach(target: i32) -> Result<i64, ()> {
    rsdb_ptrace!(PTRACE_ATTACH, target, NULL, NULL)
}

pub unsafe fn attach_wait(target: i32) -> Result<i64, ()> {
    attach(target)?;
    let ret = waitpid(target, NULL, WSTOPPED);
    if ret == -1 {
        Err(())
    }
    else {
        Ok(ret as i64)
    }
}

pub unsafe fn detach(target: i32) -> Result<i64, ()> {
    rsdb_ptrace!(PTRACE_DETACH, target, NULL, NULL)
}

pub unsafe fn cont(target: i32) -> Result<i64, ()> {
    rsdb_ptrace!(PTRACE_CONT, target, NULL, NULL)
}

pub unsafe fn sigkill(target: i32) -> Result<i64, ()> {
    let ret = rsdb_ptrace!(PTRACE_KILL, target, libc::SIGKILL, NULL);
    waitpid(target, NULL, WSTOPPED);
    ret
}

pub unsafe fn getregs(target: i32) -> Result<user_regs_struct, ()> {
    let mut data = mem::MaybeUninit::uninit();
    rsdb_ptrace!(PTRACE_GETREGS, target, NULL, 
                 data.as_mut_ptr() as *const _ as *mut c_void)?;
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