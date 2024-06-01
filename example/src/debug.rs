use bit_field::BitField;
use polyhal::TrapFrame;
use crate::kprobe::DEBUG_KPROBE_LIST;

pub fn debug_handler(trap_context:&mut TrapFrame){
    println!("<debug_handler>");
    let pc = trap_context.rip;
    let mut kporbe = DEBUG_KPROBE_LIST.lock();
    let kprobe = kporbe.get(&pc);
    if let Some(kprobe) = kprobe {
        kprobe.post_handler(&trap_context);
        let tf = trap_context.rflags.get_bit(8);
        println!("tf: {}", tf);
        println!("clear x86 single step");
        // clear single step
        trap_context.rflags.set_bit(8, false);
        // recover pc
        trap_context.rip = kprobe.next_address();
        drop(kporbe);
    }else {
        println!("There is no kprobe in pc {:#x}", pc);
        // trap_context.rip += 1; // skip ebreak instruction
        panic!("skip ebreak instruction")
    }
}