use crate::{
    pagetable::{MappingFlags, PageTable, PTE},
    PAGE_SIZE, VIRT_ADDR_START,
};

/// 汇编入口函数
///
/// 分配栈 初始化页表信息 并调到rust入口函数
#[naked]
#[no_mangle]
#[link_section = ".sigtrx.sigreturn"]
unsafe extern "C" fn _sigreturn() -> ! {
    core::arch::asm!(
        "
            li.d  $a7, 139
            syscall  0
        ",
        options(noreturn)
    )
}

#[link_section = ".data.prepage.trx1"]
static mut TRX_STEP: [[PTE; PageTable::PTE_NUM_IN_PAGE]; 2] =
    [[PTE(0); PageTable::PTE_NUM_IN_PAGE]; 2];

pub fn init() {
    unsafe {
        TRX_STEP[0][0] = PTE::from_addr(
            crate::addr::PhysAddr(_sigreturn as usize & !VIRT_ADDR_START),
            MappingFlags::URX.into(),
        );
        TRX_STEP[1][0] = PTE(TRX_STEP.as_ptr() as usize & !VIRT_ADDR_START);
    }
}

pub fn get_trx_mapping() -> usize {
    unsafe { (TRX_STEP.as_ptr() as usize + PAGE_SIZE) & !VIRT_ADDR_START }
}
