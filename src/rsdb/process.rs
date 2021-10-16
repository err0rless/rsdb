use std::path::*;
use std::fs;

use libc;

pub const KILL_SUCCESS: i32 = 0;

pub unsafe fn check_pid(pid: i32) -> i32 {
    libc::kill(pid, 0)
}

pub unsafe fn sigkill(pid: i32) -> i32 {
    libc::kill(pid, libc::SIGKILL)
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