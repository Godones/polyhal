use alloc::{
    alloc::{alloc, dealloc},
    collections::vec_deque::VecDeque,
    string::ToString,
};

use kprobe::{
    register_kprobe, register_kretprobe, unregister_kprobe, unregister_kretprobe,
    KprobeAuxiliaryOps, KprobeBuilder, KprobeManager, KprobePointList, KretprobeBuilder,
    KretprobeInstance, ProbeData, PtRegs,
};
use log::info;
use polyhal::trapframe::TrapFrame;
use spin::Mutex;

pub static KPROBE_MANAGER: Mutex<KprobeManager<Mutex<()>, FakeKprobeAuxiliaryOps>> =
    Mutex::new(KprobeManager::new());
static KPROBE_POINT_LIST: Mutex<KprobePointList<FakeKprobeAuxiliaryOps>> =
    Mutex::new(KprobePointList::new());

#[inline(never)]
fn detect_func(x: usize, y: usize) -> usize {
    let hart = 0;
    info!("detect_func: hart_id: {}, x: {}, y:{}", hart, x, y);
    x + y
}

fn pre_handler(_data: &(dyn ProbeData), pt_regs: &mut PtRegs) {
    info!("call pre_handler, the a0 is {:#x}", pt_regs.ret_value());
}

fn post_handler(_data: &(dyn ProbeData), pt_regs: &mut PtRegs) {
    info!("call post_handler, the a0 is {:#x}", pt_regs.ret_value());
}

fn fault_handler(_data: &(dyn ProbeData), pt_regs: &mut PtRegs) {
    info!("call fault_handler, the a0 is {:#x}", pt_regs.ret_value());
}

static INSTANCES: Mutex<VecDeque<KretprobeInstance>> = Mutex::new(VecDeque::new());

#[derive(Debug)]
pub struct FakeKprobeAuxiliaryOps;
impl KprobeAuxiliaryOps for FakeKprobeAuxiliaryOps {
    fn set_writeable_for_address(_address: usize, _len: usize, _writable: bool) {}

    fn alloc_executable_memory(layout: core::alloc::Layout) -> *mut u8 {
        let ptr = unsafe { alloc(layout) };
        ptr
    }

    fn dealloc_executable_memory(ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { dealloc(ptr, layout) }
    }

    fn insert_kretprobe_instance_to_task(_instance: KretprobeInstance) {
        let mut instances = INSTANCES.lock();
        instances.push_back(_instance);
    }

    fn pop_kretprobe_instance_from_task() -> KretprobeInstance {
        let mut instances = INSTANCES.lock();
        if let Some(instance) = instances.pop_front() {
            instance
        } else {
            panic!("No kretprobe instance available");
        }
    }
}

pub fn kprobe_test() {
    info!("kprobe test for [detect_func]: {:#x}", detect_func as usize);
    let kprobe_builder =
        KprobeBuilder::<FakeKprobeAuxiliaryOps>::new(None, detect_func as usize, 0, true)
            .with_pre_handler(pre_handler)
            .with_post_handler(post_handler)
            .with_fault_handler(fault_handler);

    let mut manager = KPROBE_MANAGER.lock();
    let mut kprobe_list = KPROBE_POINT_LIST.lock();

    let kprobe = register_kprobe(&mut manager, &mut kprobe_list, kprobe_builder);

    let new_pre_handler = |_data: &(dyn ProbeData), pt_regs: &mut PtRegs| {
        info!("call new pre_handler, the a0 is {:#x}", pt_regs.ret_value());
    };

    let builder2 = KprobeBuilder::<FakeKprobeAuxiliaryOps>::new(
        Some("kprobe::detect_func".to_string()),
        detect_func as usize,
        0,
        true,
    )
    .with_pre_handler(new_pre_handler)
    .with_post_handler(post_handler);

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

    detect_func(1, 2);
    info!("kprobe test end");
}

fn kretprobe_ret_handler(_data: &dyn ProbeData, regs: &mut PtRegs) {
    let ret_value = regs.ret_value();
    info!("kretprobe return value: {:#x}", ret_value);
}

pub fn kretprobe_test() {
    info!(
        "------------ kretprobe test for [detect_func]: {:#x} -------------",
        detect_func as usize
    );
    let kretprobe_builder = KretprobeBuilder::<Mutex<()>>::new(
        Some("kretprobe::detect_func".to_string()),
        detect_func as usize,
        1,
    )
    .with_entry_handler(pre_handler)
    .with_ret_handler(kretprobe_ret_handler);

    let kretprobe = register_kretprobe(
        &mut *KPROBE_MANAGER.lock(),
        &mut *KPROBE_POINT_LIST.lock(),
        kretprobe_builder,
    );

    let fake_event_callback = |_data: &dyn ProbeData, regs: &mut PtRegs| {
        info!(
            "fake event callback called, return value: {}",
            regs.ret_value()
        );
    };

    kretprobe.register_event_callback(1, fake_event_callback);

    info!(
        "install kretprobe at [detect_func]: {:#x}",
        detect_func as usize
    );
    detect_func(3, 4);

    unregister_kretprobe(
        &mut *KPROBE_MANAGER.lock(),
        &mut *KPROBE_POINT_LIST.lock(),
        kretprobe,
    );

    detect_func(3, 4);
    info!("------------ kretprobe test end -----------");
}
