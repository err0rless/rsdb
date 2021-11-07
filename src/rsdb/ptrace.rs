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
            match Errno::result(libc::ptrace($($ptrace_args), *)) {
                Ok(ret) => Ok(ret),
                Err(no) => {
                    let errstr: String = format!("ptrace: {}", no.desc());
                    println!("{}", errstr.red());
                    Err(())
                },
            }
        }
    };
}

pub unsafe fn attach(target: i32) -> Result<i64, ()> {
    rsdb_ptrace!(PTRACE_ATTACH, target, NULL, NULL)
}

pub unsafe fn attach_wait(target: i32) -> Result<i64, ()> {
    attach(target)?;
    match waitpid(target, NULL, WSTOPPED) {
        -1 => Err(()),
        i => Ok(i as i64),
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
    println!("  rax: {:16x} {:20}", regs.rax, regs.rax);
    println!("  rbx: {:16x} {:20}", regs.rbx, regs.rbx);
    println!("  rcx: {:16x} {:20}", regs.rcx, regs.rcx);
    println!("  rdx: {:16x} {:20}", regs.rdx, regs.rdx);
    println!("  rdi: {:16x} {:20}", regs.rdi, regs.rdi);
    println!("  rdx: {:16x} {:20}", regs.rdx, regs.rdx);
    println!("  r8 : {:16x} {:20}", regs.r8, regs.r8);
    println!("  r9 : {:16x} {:20}", regs.r9, regs.r9);
    println!("  r10: {:16x} {:20}", regs.r10, regs.r10);
    println!("  r11: {:16x} {:20}", regs.r11, regs.r11);
    println!("  r12: {:16x} {:20}", regs.r12, regs.r12);
    println!("  r13: {:16x} {:20}", regs.r13, regs.r13);
    println!("  r14: {:16x} {:20}", regs.r14, regs.r14);
    println!("  r15: {:16x} {:20}", regs.r15, regs.r15);
    println!("  rsp: {:16x} {:20}", regs.rsp, regs.rsp);
    println!("  rbp: {:16x} {:20}", regs.rbp, regs.rbp);
    println!("  rip: {:16x} {:20}", regs.rip, regs.rip);
}