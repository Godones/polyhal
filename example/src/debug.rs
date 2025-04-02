use kprobe::{KprobeOps, ProbeArgs};
use polyhal::trapframe::TrapFrame;

use crate::kprobe::{clear_single_step, KPROBE_MANAGER};

#[derive(Debug)]
pub struct DebugException;

impl DebugException {
    pub fn handle(frame: &mut TrapFrame) {
        Self::post_kprobe_handler(frame)
    }

    fn post_kprobe_handler(frame: &mut TrapFrame) {
        let pc = frame.debug_address();
        if let Some(kprobe_list) = KPROBE_MANAGER.lock().get_debug_list(pc) {
            for kprobe in kprobe_list {
                if kprobe.is_enabled() {
                    kprobe.call_post_handler(frame);
                    kprobe.call_event_callback(frame);
                }
            }
            let return_address = kprobe_list[0].probe_point().return_address();
            clear_single_step(frame, return_address);
        } else {
            log::debug!("There is no kprobe on pc {:#x}", pc);
        }
    }
}
