#![no_std]
#![no_main]
#![feature(core_intrinsics)]
extern crate alloc;
mod allocator;
mod debug;
mod ebreak;
mod frame;
mod kprobe;
mod logging;
mod pci;
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicU32, Ordering},
};

use frame::frame_alloc;
use polyhal::{
    addr::PhysPage,
    common::{get_fdt, get_mem_areas, PageAlloc},
    define_entry,
    instruction::shutdown,
    trap::TrapType::{self, *},
    trapframe::{TrapFrame, TrapFrameArgs},
};
use rbpf::assembler::assemble;

pub struct PageAllocImpl;

impl PageAlloc for PageAllocImpl {
    fn alloc(&self) -> PhysPage {
        frame_alloc(1)
    }

    fn dealloc(&self, ppn: PhysPage) {
        frame::frame_dealloc(ppn)
    }
}

/// kernel interrupt
#[polyhal::arch_interrupt]
fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
    // println!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
    match trap_type {
        Breakpoint => {
            #[cfg(target_arch = "riscv64")]
            {
                ctx.sepc -= 2;
            }
            #[cfg(target_arch = "loongarch64")]
            {
                ctx.era -= 4;
            }
            log::info!("BreakPoint @ {:#x}", ctx[TrapFrameArgs::SEPC]);
            let res = ebreak::EBreak::handle(ctx);
            if res.is_none() {
                log::warn!("EBreak handler returned None, continuing execution");
                #[cfg(target_arch = "riscv64")]
                {
                    ctx.sepc += 2;
                }
                #[cfg(target_arch = "loongarch64")]
                {
                    ctx.era += 4;
                }
            }
        }
        Debug => {
            log::info!("Debug @ {:#x}", ctx[TrapFrameArgs::SEPC]);
            debug::DebugException::handle(ctx);
        }
        SysCall => {
            // jump to next instruction anyway
            ctx.syscall_ok();
            log::info!("Handle a syscall");
        }
        StorePageFault(paddr) | LoadPageFault(paddr) | InstructionPageFault(paddr) => {
            log::info!("page fault: {:#x}", paddr);
        }
        IllegalInstruction(_) => {
            log::info!("illegal instruction");
        }
        Timer => {
            log::info!("Timer");
        }
        _ => {
            log::warn!("unsuspended trap type: {:?}", trap_type);
            shutdown();
        }
    }
}

static CORE_SET: AtomicU32 = AtomicU32::new(0);

#[cfg(not(target_arch = "loongarch64"))] // why loongarch64 not support assemble?
fn test_bpf_to_bpf_call() {
    println!("Testing BPF to BPF call...");
    let test_code = assemble(
        "
        mov64 r1, 0x10
        mov64 r2, 0x1
        callx 0x4
        mov64 r1, 0x1
        mov64 r2, r0
        callx 0x4
        exit
        mov64 r0, r1
        sub64 r0, r2
        exit
        mov64 r0, r2
        add64 r0, r1
        exit
        ",
    )
    .unwrap();
    let mut vm = rbpf::EbpfVmNoData::new(Some(&test_code)).unwrap();
    let vm_res = vm.execute_program().unwrap();
    assert_eq!(vm_res, 0x10);

    println!("BPF to BPF call without JIT test success!");

    #[cfg(target_arch = "x86_64")]
    {
        let mem = frame_alloc(1);
        let buf = mem.get_buffer();

        vm.set_jit_exec_memory(buf).unwrap();
        vm.jit_compile().unwrap();
        let vm_res = unsafe { vm.execute_program_jit() }.unwrap();
        assert_eq!(vm_res, 0x10);
        println!("BPF to BPF call with JIT test success!");
    }

    println!("BPF to BPF call test success!");
}

/// kernel main function, entry point.
fn main(hartid: usize) {
    if hartid != 0 {
        log::info!("Hello Other Hart: {}", hartid);
        let _ = CORE_SET.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
            Some(x | (1 << hartid))
        });
        loop {}
    }

    println!("[kernel] Hello, world!");
    allocator::init_allocator();
    log::debug!("Test Logger DEBUG!");
    log::info!("Test Logger INFO!");
    log::warn!("Test Logger WARN!");
    log::error!("Test Logger ERROR!");

    // Init page alloc for polyhal
    polyhal::common::init(&PageAllocImpl);

    get_mem_areas().into_iter().for_each(|(start, size)| {
        println!("init memory region {:#x} - {:#x}", start, start + size);
        frame::add_frame_range(start, start + size);
    });

    if let Some(fdt) = get_fdt() {
        fdt.all_nodes().for_each(|x| {
            if let Some(compatibles) = x.compatible() {
                log::debug!("Node Compatiable: {:?}", compatibles.first());
            }
        });

        log::debug!("boot args: {}", fdt.chosen().bootargs().unwrap_or(""));
    }

    // Boot another core that id is 1.
    // #[cfg(not(target_arch = "x86_64"))]
    // {
    //     use polyhal::consts::VIRT_ADDR_START;

    //     use polyhal::multicore::boot_core;
    //     use polyhal::pagetable::PAGE_SIZE;
    //     let sp = frame_alloc(16);
    //     boot_core(1, (sp.to_addr() | VIRT_ADDR_START) + 16 * PAGE_SIZE);
    //     // Waiting for Core Booting
    //     while CORE_SET.fetch_and(1 << 1, Ordering::SeqCst) == 0 {}
    //     log::info!("Core 1 Has Booted successfully!");
    // }

    unsafe { core::intrinsics::breakpoint() }

    // log::info!("test a simple break");

    // Test BreakPoint Interrupt
    kprobe::kprobe_test();
    kprobe::kretprobe_test();

    #[cfg(not(target_arch = "loongarch64"))]
    test_bpf_to_bpf_call();

    crate::pci::init();

    // loop {
    //     if let Some(c) = DebugConsole::getchar() {
    //         DebugConsole::putchar(c);
    //     }
    // }

    log::info!("Run END. Shutdown successfully.");
    shutdown();
}

define_entry!(main);
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log::error!(
            "[kernel] Panicked at {}:{} \n\t{}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        log::error!("[kernel] Panicked: {}", info.message());
    }
    shutdown()
}
