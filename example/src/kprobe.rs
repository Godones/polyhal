use alloc::string::ToString;

use kprobe::{
    register_kprobe, unregister_kprobe, KprobeBuilder, KprobeManager, KprobePointList, ProbeArgs,
};
use log::info;
use polyhal::trapframe::{TrapFrame, TrapFrameArgs};
use spin::Mutex;

pub static KPROBE_MANAGER: Mutex<KprobeManager> = Mutex::new(KprobeManager::new());
static KPROBE_POINT_LIST: Mutex<KprobePointList> = Mutex::new(KprobePointList::new());

pub fn setup_single_step(frame: &mut TrapFrame, step_addr: usize) {
    #[cfg(target_arch = "riscv64")]
    {
        frame.sepc = step_addr;
    }
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 does not need to set the sepc
        // frame.sepc = step_addr;
        frame.rip = step_addr;
        frame.rflags |= 0x100; // Set the TF flag
    }
}

pub fn clear_single_step(frame: &mut TrapFrame, return_addr: usize) {
    #[cfg(target_arch = "riscv64")]
    {
        frame.sepc = return_addr;
    }
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 does not need to set the sepc
        // frame.sepc = return_addr;
        frame.rip = return_addr;
        frame.rflags &= !0x100; // Clear the TF flag
    }
}

#[inline(never)]
fn detect_func(x: usize, y: usize) -> usize {
    let hart = 0;
    info!("detect_func: hart_id: {}, x: {}, y:{}", hart, x, y);
    hart
}

fn pre_handler(regs: &dyn ProbeArgs) {
    let pt_regs = regs.as_any().downcast_ref::<TrapFrame>().unwrap();
    info!(
        "call pre_handler, the sp is {:#x}",
        pt_regs[TrapFrameArgs::SP]
    );
}

fn post_handler(regs: &dyn ProbeArgs) {
    let pt_regs = regs.as_any().downcast_ref::<TrapFrame>().unwrap();
    info!(
        "call post_handler, the sp is {:#x}",
        pt_regs[TrapFrameArgs::SP]
    );
}

fn fault_handler(regs: &dyn ProbeArgs) {
    let pt_regs = regs.as_any().downcast_ref::<TrapFrame>().unwrap();
    info!(
        "call fault_handler, the sp is {:#x}",
        pt_regs[TrapFrameArgs::SP]
    );
}

pub fn kprobe_test() {
    info!("kprobe test for [detect_func]: {:#x}", detect_func as usize);
    let kprobe_builder = KprobeBuilder::new(
        None,
        detect_func as usize,
        0,
        pre_handler,
        post_handler,
        true,
    )
    .with_fault_handler(fault_handler);

    let mut manager = KPROBE_MANAGER.lock();
    let mut kprobe_list = KPROBE_POINT_LIST.lock();

    let kprobe = register_kprobe(&mut manager, &mut kprobe_list, kprobe_builder);

    let new_pre_handler = |regs: &dyn ProbeArgs| {
        let pt_regs = regs.as_any().downcast_ref::<TrapFrame>().unwrap();
        info!(
            "call new pre_handler, the sp is {:#x}",
            pt_regs as *const _ as usize
        );
    };

    let builder2 = KprobeBuilder::new(
        Some("kprobe::detect_func".to_string()),
        detect_func as usize,
        0,
        new_pre_handler,
        post_handler,
        true,
    );
    let kprobe2 = register_kprobe(&mut manager, &mut kprobe_list, builder2);
    drop(manager);
    drop(kprobe_list);
    info!(
        "install 2 kprobes at [detect_func]: {:#x}",
        detect_func as usize
    );
    detect_func(1, 2);

    let mut manager = KPROBE_MANAGER.lock();
    let mut kprobe_list = KPROBE_POINT_LIST.lock();
    unregister_kprobe(&mut manager, &mut kprobe_list, kprobe);
    unregister_kprobe(&mut manager, &mut kprobe_list, kprobe2);
    info!(
        "uninstall 2 kprobes at [detect_func]: {:#x}",
        detect_func as usize
    );
    detect_func(1, 2);
    info!("kprobe test end");
}
