use kprobe::{KprobeOps, ProbeArgs};
use polyhal::trapframe::TrapFrame;

use crate::{
    debug::DebugException,
    kprobe::{setup_single_step, KPROBE_MANAGER},
};

#[derive(Debug)]
pub struct EBreak;

impl EBreak {
    pub fn handle(frame: &mut TrapFrame) {
        Self::kprobe_handler(frame)
    }
    fn kprobe_handler(frame: &mut TrapFrame) {
        let break_addr = frame.break_address();
        // log::debug!("EBreak: break_addr: {:#x}", break_addr);
        let guard = KPROBE_MANAGER.lock();
        let kprobe_list = guard.get_break_list(break_addr);
        if let Some(kprobe_list) = kprobe_list {
            for kprobe in kprobe_list {
                if kprobe.is_enabled() {
                    kprobe.call_pre_handler(frame);
                }
            }
            let single_step_address = kprobe_list[0].probe_point().single_step_address();
            // setup_single_step
            setup_single_step(frame, single_step_address);
        } else {
            // For some architectures, they do not support single step execution,
            // and we need to use breakpoint exceptions to simulate
            drop(guard);
            DebugException::handle(frame);
        }
    }
}
