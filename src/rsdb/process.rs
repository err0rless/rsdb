use std::{os::unix::prelude::CommandExt, path::*};
use linux_personality::personality;

use super::procfs;

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
    pub fn new() -> Proc {
        Proc { 
            target: -1, 
            file: PathBuf::new(),
            cmdline: String::from(""), 
            exe: PathBuf::new(), 
            cwd: PathBuf::new(),
            maps: String::from("") 
        }
    }

    pub fn from(&mut self, pid: i32) {
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
    }

    pub fn file_available(&self) -> bool {
        !self.file.to_str().unwrap().is_empty()
    }

    pub fn get_pid(&self) -> nix::unistd::Pid {
        PidType::from_raw(self.target)
    }
    
    // Spawn, attach and wait
    pub fn spawn_file(&mut self) -> i32 {
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
                std::process::Command::new(self.file.as_path())
                    .exec();

                // this is child process
                -1
            },
            Ok(nix::unistd::ForkResult::Parent { child }) => {
                self.target = child.as_raw();
                println!("Successfully spawned a child with");
                println!("  path: {}", self.file.canonicalize().unwrap().display());
                println!("  pid : {}", self.target);

                child.as_raw()
            },
            Err(err) => {
                println!("Fork failed with error: {}", err);
                -1
            }
        }
    }

    pub fn available(&self) -> bool {
        self.target != -1
    }
 
    pub fn update(&mut self) {
        self.from(self.target);
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

    pub fn release(&mut self) {
        use colored::Colorize;
        println!("{}{}", "Releasing process: ".red(), self.target);
        self.target = -1;
        self.cmdline.clear();
        self.exe.clear();
        self.cwd.clear();
    }
}
