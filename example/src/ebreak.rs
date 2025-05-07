use kprobe::kprobe_handler_from_break;
use polyhal::trapframe::TrapFrame;

use crate::kprobe::KPROBE_MANAGER;

#[derive(Debug)]
pub struct EBreak;

impl EBreak {
    pub fn handle(frame: &mut TrapFrame) {
        Self::kprobe_handler(frame)
    }
    fn kprobe_handler(frame: &mut TrapFrame) {
        let mut manager = KPROBE_MANAGER.lock();
        kprobe_handler_from_break(&mut manager, frame);
    }
}
