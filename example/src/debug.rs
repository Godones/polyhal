use kprobe::kprobe_handler_from_debug;
use polyhal::trapframe::TrapFrame;

use crate::kprobe::KPROBE_MANAGER;

#[derive(Debug)]
pub struct DebugException;

impl DebugException {
    pub fn handle(frame: &mut TrapFrame) {
        Self::post_kprobe_handler(frame)
    }

    fn post_kprobe_handler(frame: &mut TrapFrame) {
        let mut manager = KPROBE_MANAGER.lock();
        kprobe_handler_from_debug(&mut manager, frame);
    }
}
