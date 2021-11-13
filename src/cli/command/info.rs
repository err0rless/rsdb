use super::*;

pub fn regs(proc: &mut process::Proc) -> MainLoopAction {
    proc.dump_regs();
    MainLoopAction::None
}

pub fn proc(proc: &mut process::Proc) -> MainLoopAction {
    proc.update();
    proc.dump();
    MainLoopAction::None
}