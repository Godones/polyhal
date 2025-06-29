use kprobe::kprobe_handler_from_debug;
use polyhal::trapframe::TrapFrame;

use crate::{
    ebreak::{pt_regs_from_trapframe, pt_regs_to_trapframe},
    kprobe::KPROBE_MANAGER,
};

#[derive(Debug)]
pub struct DebugException;

impl DebugException {
    pub fn handle(frame: &mut TrapFrame) -> Option<()> {
        Self::post_kprobe_handler(frame)
    }

    fn post_kprobe_handler(frame: &mut TrapFrame) -> Option<()> {
        let mut manager = KPROBE_MANAGER.lock();
        let mut pt_regs = pt_regs_from_trapframe(frame);
        let res = kprobe_handler_from_debug(&mut manager, &mut pt_regs);
        pt_regs_to_trapframe(frame, &pt_regs);
        res
    }
}
