use super::*;

pub fn regs(proc: &mut process::Proc) -> MainLoopAction {
    proc.dump_regs();
    MainLoopAction::None
}

pub fn proc(sess: &mut session::Session) -> MainLoopAction {
    sess.proc.update();
    sess.proc.dump();
    MainLoopAction::None
}