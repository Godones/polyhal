use polyhal::TrapFrame;
use crate::kprobe::BREAK_KPROBE_LIST;

pub fn ebreak_handler(trap_context:&mut TrapFrame){
    println!("<ebreak_handler>");
    let pc = trap_context.rip -1;
    let mut kporbe = BREAK_KPROBE_LIST.lock();
    let kprobe = kporbe.get_mut(&pc);
    if let Some(kprobe) = kprobe {
        kprobe.pre_handler(&trap_context);
        // set single step
        println!("set x86 single step");
        trap_context.rflags |= 0x100;
        let old_instruction = kprobe.old_inst();
        println!("old_instruction: {:x?}, address: {:#x}", old_instruction, old_instruction.as_ptr() as usize);
        // single execute old instruction
        trap_context.rip  = old_instruction.as_ptr() as usize;
        drop(kporbe);
    }else {
        println!("There is no kprobe in pc {:#x}", pc);
        panic!("skip ebreak instruction")
    }
}