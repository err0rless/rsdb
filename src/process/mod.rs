use std::path::*;
use std::os::unix::prelude::CommandExt;
use libc::user_regs_struct;
use linux_personality::personality;

use crate::traits::*;
use crate::ptrace;

pub mod procfs;

pub type PidType = nix::unistd::Pid;

pub struct Proc {
    pub target: i32,
    pub file: PathBuf,
    cmdline: String,
    exe: PathBuf,
    cwd: PathBuf,
    maps: String,
}

impl Proc {
    pub fn new() -> Self {
        Proc { 
            target: -1, 
            file: PathBuf::new(),
            cmdline: String::from(""), 
            exe: PathBuf::new(), 
            cwd: PathBuf::new(),
            maps: String::from(""),
        }
    }

    pub fn set(&mut self, pid: i32) -> Result<i32, ()> {
        if self.valid() {
            println!("Failed to process::set => Process not released {}", self.target);
            return Err(());
        }

        self.target = pid;
        self.cmdline = match procfs::get_proc_cmdline(pid) {
            Ok(cmdline) => cmdline,
            Err(_) => String::from(""),
        };
        self.exe = match procfs::get_proc_exe(pid) {
            Ok(exe) => exe,
            Err(_) => PathBuf::new(),
        };
        self.cwd = match procfs::get_proc_cwd(pid) {
            Ok(cwd) => cwd,
            Err(_) => PathBuf::new(),
        };
        self.maps = match procfs::get_proc_maps(pid) {
            Ok(maps) => maps,
            Err(_) => String::from(""),
        };
        Ok(pid)
    }

    pub fn get_pid(&self) -> nix::unistd::Pid {
        PidType::from_raw(self.target)
    }
    
    pub fn update(&mut self) {
        self.cmdline = match procfs::get_proc_cmdline(self.target) {
            Ok(cmdline) => cmdline,
            Err(_) => String::from(""),
        };
        self.exe = match procfs::get_proc_exe(self.target) {
            Ok(exe) => exe,
            Err(_) => PathBuf::new(),
        };
        self.cwd = match procfs::get_proc_cwd(self.target) {
            Ok(cwd) => cwd,
            Err(_) => PathBuf::new(),
        };
        self.maps = match procfs::get_proc_maps(self.target) {
            Ok(maps) => maps,
            Err(_) => String::from(""),
        };
    }

    pub fn dump(&self) {
        println!("pid = {}", self.target);
        println!("cmdline = '{}'", self.cmdline);
        println!("exe = '{}'", self.exe.display());
        println!("cwd = '{}'", self.cwd.display());
    }

    pub fn dump_maps(&self) {
        println!("{}", self.maps);
    }

    pub fn getregs(&self) -> Result<user_regs_struct, () >{
        unsafe { ptrace::getregs(self.target) }
    }

    pub fn getreg(&self, regname: &str) -> Result<u64, ()> {
        match self.getregs() {
            Ok(regs) => {
                match regname {
                    "rax" => Ok(regs.rax),
                    "rbx" => Ok(regs.rbx),
                    "rcx" => Ok(regs.rcx),
                    "rdx" => Ok(regs.rdx),
                    "r8"  => Ok(regs.r8),
                    "r9"  => Ok(regs.r9),
                    "r10" => Ok(regs.r10),
                    "r11" => Ok(regs.r11),
                    "r12" => Ok(regs.r12),
                    "r13" => Ok(regs.r13),
                    "r14" => Ok(regs.r14),
                    "r15" => Ok(regs.r15),
                    "rsp" => Ok(regs.rsp),
                    "rbp" => Ok(regs.rbp),
                    "rip" => Ok(regs.rip),
                    _ => Err(()),
                }
            }
            Err(_) => Err(()),
        }
    }

    pub fn dump_regs(&self) {
        ptrace::dumpregs(&self.getregs().unwrap());
    }

    pub fn release(&mut self) {
        use colored::Colorize;
        println!("{}{}", "Releasing process: ".red(), self.target);
        
        self.target = -1;
        self.cmdline.clear();
        self.exe.clear();
        self.cwd.clear();
    }
}

impl Valid for Proc {
    fn valid(&self) -> bool { self.target != 1 }
}

// Spawn, attach and wait
pub fn spawn_file(file: &PathBuf) -> i32 {
    match unsafe{ nix::unistd::fork() } {
        Ok(nix::unistd::ForkResult::Child) => {
            // ptrace(PTRACE_TRACEME, ...);
            nix::sys::ptrace::traceme().unwrap_or_else(|e| {
                println!("ptrace::traceme() failed with code {}", e);
            });

            // disable ASLR
            personality(linux_personality::ADDR_NO_RANDOMIZE).unwrap_or_else(|_| {
                println!("failed to disable ASLR");
                linux_personality::Personality::empty()
            });

            // run executable on this process
            std::process::Command::new(file.as_path())
                .exec();

            // this is child process
            -1
        },
        Ok(nix::unistd::ForkResult::Parent { child }) => {
            println!("Successfully spawned a child with");
            println!("  path: {}", file.canonicalize().unwrap().display());
            println!("  pid : {}", child.as_raw());

            child.as_raw()
        },
        Err(err) => {
            println!("Fork failed with error: {}", err);
            -1
        }
    }
}