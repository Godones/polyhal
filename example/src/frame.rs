use buddy_system_allocator::LockedFrameAllocator;
use polyhal::{addr::PhysPage, pagetable::PAGE_SIZE};
use spin::Lazy;

static LOCK_FRAME_ALLOCATOR: Lazy<LockedFrameAllocator<32>> =
    Lazy::new(|| LockedFrameAllocator::new());

pub fn add_frame_range(mm_start: usize, mm_end: usize) {
    extern "C" {
        fn end();
    }
    let mm_start = if mm_start <= mm_end && mm_end > end as usize {
        (end as usize + PAGE_SIZE - 1) / PAGE_SIZE
    } else {
        mm_start / PAGE_SIZE
    };
    let mm_end = mm_end / PAGE_SIZE;
    LOCK_FRAME_ALLOCATOR.lock().add_frame(mm_start, mm_end);
}

pub fn frame_alloc(count: usize) -> PhysPage {
    let ppn = LOCK_FRAME_ALLOCATOR
        .lock()
        .alloc(count)
        .expect("can't find memory page");
    PhysPage::new(ppn)
}

pub fn frame_dealloc(ppn: PhysPage) {
    LOCK_FRAME_ALLOCATOR.lock().dealloc(ppn.as_num(), 1);
}
