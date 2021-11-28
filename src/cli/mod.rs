use std::iter::*;
use regex::Regex;
use colored::*;

use crate::{session, process::*, traits::*};
use command::MainLoopAction;

pub mod command;

macro_rules! continue_if {
    ($cond:expr) => {
        if $cond {
            return MainLoopAction::Continue;
        }
    };
    ($cond:expr, $msg:tt) => {
        if $cond {
            println!("{}", $msg.red());
            return MainLoopAction::Continue;
        }
    };
}

fn rsdb_help() -> MainLoopAction {
    println!("{}", "rsdb: Linux Debugger written in Rust".bright_yellow());
    println!("  help | ? => Print help");
    println!("  attach [PID | Package name] => attach to the prcess");
    println!("    e.g) {} or {}", "attach 31337".bright_yellow(), "attach com.test.package".bright_yellow());
    println!("  detach => detach from the process");
    println!("  run | r => run the process only if --file argument given");
    println!("  info => info [Subcommand]");
    println!("    regs => show registers");
    println!("    proc => show process informations");
    println!("  vmmap | maps => show memory maps of the process");
    println!("  kill => send signal to the attached process");
    println!("  exit | quit => Exit rsdb");
    MainLoopAction::None
}

pub fn rsdb_main(session: &mut session::Session, buffer: &String) -> MainLoopAction {
    let re = Regex::new(r"\s+").unwrap();
    let fullcmd = re.replace_all(buffer.trim(), " ");
    let commands = Vec::from_iter(fullcmd.split(" ").map(String::from));
    let command = &commands[0];

    match command.as_str() {
        "attach" => {
            continue_if!(commands.len() != 2, "Usage: attach [PID | Package/Process name]");
            continue_if!(session.proc.valid(), "rsdb is already holding the process, detach first");
            
            let process = &commands[1];
            let new_target = match process.parse::<i32>() {
                Ok(pid) => pid,
                Err(_) => procfs::findpid(process),
            };
            continue_if!(unsafe { !procfs::check_pid(new_target) }, 
                         "pid doesn't exist, check again");
            command::attach(session, new_target)
        },
        "detach" => {
            continue_if!(session.invalid(), "No process has been attached");
            command::detach(session.mut_proc())
        },
        "continue" | "c" => {
            continue_if!(session.invalid(), "No process has been attached");
            command::cont(session.mut_proc())
        },
        "run" | "r" => {
            continue_if!(session.valid(), "rsdb is already holding the process, detach first");
            continue_if!(session.elf.is_none(), "File is not available!");
            command::run(session)
        },
        "info" => {
            continue_if!(commands.len() != 2, "Usage: info [Subcommand], help for more details");
            match commands[1].as_str() {
                "regs" | "r" => {
                    continue_if!(session.invalid(), "No process has been attached");
                    command::info::regs(session.mut_proc());
                },
                "proc" => {
                    continue_if!(session.invalid(), "No process has been attached");
                    command::info::proc(session.mut_proc());
                },
                subcommand => println!("{}'{}'", "info: invalid subcommand: ".red(), subcommand),
            }
            MainLoopAction::None
        },
        "vmmap" | "maps" => {
            continue_if!(session.invalid(), "No process has been attached");
            command::vmmap(session.mut_proc())
        },
        "kill" => {
            continue_if!(commands.len() != 1, "Usage: kill");
            continue_if!(session.invalid(), "No process has been attached");
            command::kill(session.mut_proc())
        },
        "exit" | "quit" | "q" => command::quit(session.mut_proc()),
        "help" | "?" => rsdb_help(),
        "" => MainLoopAction::None,
        invalid_cmd => {
            println!("{}: {}", "Invalid command".red(), invalid_cmd);
            MainLoopAction::None
        },
    }
}
