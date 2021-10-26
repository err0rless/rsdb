use std::path::*;
use std::fs;
use core::fmt;

use libc;

const KILL_SUCCESS: i32 = 0;

pub struct Proc {
    pub target: i32,
    pub cmdline: String,
    pub exe: PathBuf,
    pub cwd: PathBuf,
}

impl Proc {
    pub fn init_with_pid(&mut self, pid: i32) {
        self.target = pid;
        if let Ok(cmdline) = get_proc_cmdline(pid) {
            self.cmdline = cmdline;
        }
        if let Ok(exe) = get_proc_exe(pid) {
            self.exe = exe;
        }
        if let Ok(cwd) = get_proc_cwd(pid) {
            self.cwd = cwd;
        }
    }

    pub fn dump(&self) {
        println!("pid = {}", self.target);
        println!("cmdline = '{}'", self.cmdline);
        println!("exe = '{}'", self.exe.display());
        println!("cwd = '{}'", self.cwd.display());
    }

    pub fn clear(&mut self) {
        self.target = -1;
        self.cmdline.clear();
        self.exe.clear();
        self.cwd.clear();
    }
}

impl fmt::Display for Proc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pid = {}\ncmdline = '{}'\nexe = '{}'\ncwd = '{}'", 
            self.target, self.cmdline, self.exe.display(), self.cwd.display())
    }
}

pub fn get_proc_cmdline(target: i32) -> Result<String, ()> {
    let mut path = PathBuf::from("/proc");
    path.push(target.to_string());
    path.push("cmdline");
    if !path.exists() {
        println!("Cannot open file: {}", path.display());
        return Err(());
    }
    match fs::read_to_string(&path) {
        Ok(cmd) => Ok(cmd),
        Err(_) => Err(()),
    }
}

pub fn get_proc_exe(target: i32) -> Result<PathBuf, ()> {
    let mut path = PathBuf::from("/proc");
    path.push(target.to_string());
    path.push("exe");
    match fs::canonicalize(path) {
        Ok(dest) => Ok(dest),
        Err(_) => Err(()),
    }
}

pub fn get_proc_cwd(target: i32) -> Result<PathBuf, ()> {
    let mut path = PathBuf::from("/proc");
    path.push(target.to_string());
    path.push("cwd");
    match path.read_link() {
        Ok(dest) => Ok(dest),
        Err(_) => Err(()),
    }
}

pub fn get_proc_maps(target: i32) -> Result<String, ()> {
    let mut path = PathBuf::from("/proc");
    path.push(target.to_string());
    path.push("maps");
    if !path.exists() {
        println!("Cannot open file: {}", path.display());
        return Err(());
    }
    match fs::read_to_string(&path) {
        Ok(cmd) => Ok(cmd),
        Err(_) => Err(()),
    }
}

pub unsafe fn check_pid(pid: i32) -> bool {
    libc::kill(pid, 0) == KILL_SUCCESS
}

pub fn findpid(from: &str) -> i32 {
    let proc = fs::read_dir("/proc");
    if let Err(_) = proc {
        panic!("Failed to open '/proc'");
    }
    
    for path in proc.unwrap() {
        if path.is_err() { continue; }

        // "/proc/{PID}/cmdline"
        let newpath: PathBuf = {
            let mut innerpath = path.unwrap().path();
            innerpath.push("cmdline");
            innerpath
        };
        if let Ok(cmd) = fs::read_to_string(&newpath) {
            let cmd_first = cmd.split(" ").nth(0);
            if let Some(executable) = cmd_first {
                if !executable.contains(from) {
                    continue;
                }
                let parent = newpath.parent().unwrap();
                let filename = {
                    let filename = parent.file_name().unwrap();
                    filename.to_str().unwrap()
                };
                if let Ok(n) = filename.parse::<i32>() {
                    return n
                }
            }
        }
    }

    println!("rsdb failed to find process name: '{}'", from);
    -1
}