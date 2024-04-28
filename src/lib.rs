#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(stdsimd)]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts_mut)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![cfg_attr(target_arch = "aarch64", feature(const_option))]

/// This is a crate to help you supporting multiple platforms.
///
/// If you want to use this crate, you should implement the following trait in your code.
///
/// ```rust
/// use arch::api::ArchInterface;
///
/// pub struct ArchInterfaceImpl;
///
/// #[crate_interface::impl_interface]
/// impl ArchInterface for ArchInterfaceImpl {
///     /// Init allocator
///     fn init_allocator() {}
///     /// kernel interrupt
///     fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {}
///     /// init log
///     fn init_logging() {}
///     /// add a memory region
///     fn add_memory_region(start: usize, end: usize) {}
///     /// kernel main function, entry point.
///     fn main(hartid: usize) {}
///     /// Alloc a persistent memory page.
///     fn frame_alloc_persist() -> PhysPage {}
///     /// Unalloc a persistent memory page
///     fn frame_unalloc(ppn: PhysPage) {}
///     /// Preprare drivers.
///     fn prepare_drivers() {}
///     /// Try to add device through FdtNode
///     fn try_to_add_device(_fdt_node: &FdtNode) {}
/// }
/// ```
///
/// The main(hardid: usize) is the entry point.
///
/// You can find details in the example.
///
extern crate alloc;

#[macro_use]
extern crate log;

pub mod addr;
pub mod api;
#[macro_use]
pub mod consts;
pub mod debug;
pub mod irq;
#[cfg(feature = "multicore")]
pub mod multicore;
pub mod once;
pub mod pagetable;
pub mod time;
use core::mem::size_of;

use addr::PhysPage;
use alloc::vec::Vec;

use consts::STACK_SIZE;
use fdt::Fdt;
use once::LazyInit;
use pagetable::PageTable;
pub use percpu;

#[cfg_attr(target_arch = "riscv64", path = "riscv64/mod.rs")]
#[cfg_attr(target_arch = "aarch64", path = "aarch64/mod.rs")]
#[cfg_attr(target_arch = "x86_64", path = "x86_64/mod.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "loongarch64/mod.rs")]
mod currrent_arch;

/// Trap Frame
pub use currrent_arch::TrapFrame;

pub use currrent_arch::*;

pub use arch_macro::{arch_entry, arch_interrupt};

pub const PAGE_SIZE: usize = PageTable::PAGE_SIZE;
pub const USER_VADDR_END: usize = PageTable::USER_VADDR_END;

/// Kernel Context Arg Type.
///
/// Using this by Index and IndexMut trait bound on KContext.
#[derive(Debug)]
#[cfg(feature = "kcontext")]
pub enum KContextArgs {
    /// Kernel Stack Pointer
    KSP,
    /// Kernel Thread Pointer
    KTP,
    /// Kernel Program Counter
    KPC,
}

/// Trap Frame Arg Type
/// 
/// Using this by Index and IndexMut trait bound on TrapFrame
#[derive(Debug)]
pub enum TrapFrameArgs {
    SEPC,
    RA,
    SP,
    RET,
    ARG0,
    ARG1,
    ARG2,
    TLS,
    SYSCALL,
}

#[derive(Debug, Clone, Copy)]
pub enum TrapType {
    Breakpoint,
    UserEnvCall,
    Time,
    Unknown,
    SupervisorExternal,
    StorePageFault(usize),
    LoadPageFault(usize),
    InstructionPageFault(usize),
    IllegalInstruction(usize),
}

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

pub(crate) fn clear_bss() {
    extern "C" {
        fn _sbss();
        fn _ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            _sbss as usize as *mut u128,
            (_ebss as usize - _sbss as usize) / size_of::<u128>(),
        )
        .fill(0);
    }
}

pub trait PageAlloc: Sync {
    fn alloc(&self) -> PhysPage;
    fn dealloc(&self, ppn: PhysPage);
}

static PAGE_ALLOC: LazyInit<&dyn PageAlloc> = LazyInit::new();

/// Init arch with page allocator, like log crate
/// Please initialize the allocator before calling this function.
pub fn init(page_alloc: &'static dyn PageAlloc) {
    PAGE_ALLOC.init_by(page_alloc);

    // Init current architecture
    currrent_arch::arch_init();
}

/// Store the number of cpu, this will fill up by startup function.
pub(crate) static CPU_NUM: LazyInit<usize> = LazyInit::new();

/// Store the memory area, this will fill up by the arch_init() function in each architecture.
pub(crate) static MEM_AREA: LazyInit<Vec<(usize, usize)>> = LazyInit::new();

/// Store the DTB_area, this will fill up by the arch_init() function in each architecture
static DTB_BIN: LazyInit<Vec<u8>> = LazyInit::new();

/// Get the memory area, this function should be called after initialization
pub fn get_mem_areas() -> Vec<(usize, usize)> {
    MEM_AREA.clone()
}

/// Get the fdt
pub fn get_fdt() -> Option<Fdt<'static>> {
    Fdt::new(&DTB_BIN).ok()
}

/// Get the number of cpus
pub fn get_cpu_num() -> usize {
    *CPU_NUM
}
