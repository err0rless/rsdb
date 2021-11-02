use std::path::*;
use super::procfs;

pub struct Proc {
    pub target: i32,
    cmdline: String,
    exe: PathBuf,
    cwd: PathBuf,
    maps: String,
}

impl Proc {
    pub fn new() -> Proc {
        Proc { 
            target: -1, 
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
