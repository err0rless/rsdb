use std::path::*;
use nix::NixPath;

use super::{procfs, ptrace};

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
        !self.file.is_empty()
    }
    
    // Spawn, attach and wait
    pub fn spawn_file(&mut self) {
        let spawn_child = 
            std::process::Command::new(self.file.as_path())
                .spawn();
        match spawn_child {
            Ok(child) => {
                self.target = child.id() as i32;

                println!("Successfully spawned a child with");
                println!("  path: {}", self.file.canonicalize().unwrap().display());
                println!("  pid : {}", self.target);

                unsafe { let _ = ptrace::attach_wait(self.target); };
            },
            Err(e) => println!("Failed to spawn: {}", e),
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

    pub fn clear(&mut self) {
        self.target = -1;
        self.cmdline.clear();
        self.exe.clear();
        self.cwd.clear();
    }
}
