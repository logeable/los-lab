use core::ops::Range;

use crate::{config::KERNEL_HEAP_SIZE, println};
use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layot = {:?}", layout);
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init(_mem_range: Range<usize>) {
    unsafe {
        #[allow(static_mut_refs)]
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE)
    };
}

pub fn kernel_heap_stats() {
    println!(
        "actual: {0}/{0:#x}",
        HEAP_ALLOCATOR.lock().stats_alloc_actual()
    );

    println!(
        "requested: {0}/{0:#x}",
        HEAP_ALLOCATOR.lock().stats_alloc_user()
    );

    println!(
        "total: {0}/{0:#x}",
        HEAP_ALLOCATOR.lock().stats_total_bytes()
    );
}
