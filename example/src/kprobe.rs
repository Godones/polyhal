use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::fmt::Debug;
use spin::Mutex;
use polyhal::hart_id;
use yaxpeax_arch::LengthedInstruction;
use polyhal::addr::VirtAddr;

pub static BREAK_KPROBE_LIST: Mutex<BTreeMap<usize, Arc<Kprobe>>> = Mutex::new(BTreeMap::new());
pub static DEBUG_KPROBE_LIST: Mutex<BTreeMap<usize, Arc<Kprobe>>> = Mutex::new(BTreeMap::new());
type PtRegs = polyhal::TrapFrame;

pub struct Kprobe {
    symbol: String,
    symbol_addr: usize,
    offset: usize,
    old_instruction: [u8;15],
    old_instruction_len: usize,
    pre_handler: Box<dyn Fn(&PtRegs)>,
    post_handler: Box<dyn Fn(&PtRegs)>,
    fault_handler: Box<dyn Fn(&PtRegs)>,
    inst_tmp: [u8; 15],
}



impl Debug for Kprobe {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Kprobe")
            .field("symbol", &self.symbol)
            .field("offset", &self.offset)
            .finish()
    }
}

unsafe impl Send for Kprobe {}
unsafe impl Sync for Kprobe {}


impl Kprobe {
    pub fn new(
        symbol: String,
        symbol_addr: usize,
        offset: usize,
        pre_handler: Box<dyn Fn(&PtRegs)>,
        post_handler: Box<dyn Fn(&PtRegs)>,
        fault_handler: Box<dyn Fn(&PtRegs)>,
    ) -> Self {
        Kprobe {
            symbol,
            symbol_addr,
            offset,
            old_instruction: [ 0;15],
            old_instruction_len: 0,
            pre_handler,
            post_handler,
            fault_handler,
            inst_tmp: [0;15],
        }
    }

    pub fn install(&mut self) {
        let address = self.symbol_addr + self.offset;
        let max_instruction_size = 15; // x86_64 max instruction length
        let mut inst_tmp = [0u8; 15];
        unsafe { core::ptr::copy(address as *const u8, inst_tmp.as_mut_ptr(), max_instruction_size); }

        let decoder = yaxpeax_x86::amd64::InstDecoder::default();

        let inst = decoder.decode_slice(&inst_tmp).unwrap();
        println!("inst: {:?}", inst.to_string());
        let len = inst.len().to_const();
        println!("inst.len: {:?}", len);

        self.old_instruction = inst_tmp;
        self.old_instruction_len = len as usize;

        let ebreak_inst = 0xcc; // x86_64: 0xcc
        unsafe {
            core::ptr::write_volatile(address as *mut u8, ebreak_inst);
        }
        polyhal::pagetable::TLB::flush_vaddr(VirtAddr::new(address));
        println!(
            "Kprobe::install: address: {:#x}, func_name: {}",
            address, self.symbol
        );
    }


    pub fn uninstall(&mut self) {
        let address = self.symbol_addr + self.offset;
        unsafe {
            core::ptr::copy(self.old_instruction.as_ptr(), address as *mut u8, self.old_instruction_len);
        }
        let decoder = yaxpeax_x86::amd64::InstDecoder::default();
        let inst = decoder.decode_slice(&self.old_instruction).unwrap();
        println!("inst: {:?}", inst.to_string());
        println!(
            "Kprobe::uninstall: address: {:#x}, old_instruction: {:?}",
            address, inst
        );
    }

    pub fn address(&self) -> usize {
        self.symbol_addr + self.offset
    }

    pub fn next_address(&self) -> usize {
        self.symbol_addr + self.offset + self.old_instruction_len
    }

    pub fn pre_handler(&self, regs: &PtRegs) {
        (self.pre_handler)(regs);
    }

    pub fn post_handler(&self, regs: &PtRegs) {
        (self.post_handler)(regs);
    }

    pub fn fault_handler(&self, regs: &PtRegs) {
        (self.fault_handler)(regs);
    }

    pub fn old_inst(&self) -> &[u8;15] {
        &self.old_instruction
    }

    pub fn debug_address(&self) -> usize {
        self.old_instruction_len + self.old_instruction.as_ptr() as usize
    }
}

#[inline(never)]
#[no_mangle]
pub fn detect_func(x: usize,y:usize) -> usize {
    let hart = hart_id();
    println!("detect_func: hart_id: {}, x: {}, y:{}", hart, x,y);
    hart
}

#[no_mangle]
pub fn test_kprobe() {
    let pre_handler = Box::new(|regs: &PtRegs| {
        println!("call pre_handler");
    });
    let post_handler = Box::new(|regs: &PtRegs| {
        println!("call post_handler");
    });
    let fault_handler = Box::new(|regs: &PtRegs| {
        println!("call fault_handler");
    });
    let mut kprobe = Kprobe::new(
        "detect_func".to_string(),
        detect_func as usize,
        0,
        pre_handler,
        post_handler,
        fault_handler,
    );

    kprobe.install();
    let kprobe = Arc::new(kprobe);
    BREAK_KPROBE_LIST.lock().insert(detect_func as usize, kprobe.clone());
    let debug_address = kprobe.debug_address();
    DEBUG_KPROBE_LIST.lock().insert(debug_address, kprobe);

    detect_func(1,2);
}
