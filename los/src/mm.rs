use lazy_static::lazy_static;
use spin::Mutex;

mod address;
mod frame_allocator;
mod heap;
mod memory_space;
mod page_table;

lazy_static! {
    pub static ref KERNEL_MEMORY_SPACE: Mutex<memory_space::MemorySpace> =
        Mutex::new(memory_space::MemorySpace::new_kernel());
}
pub fn init() {
    heap::init();
    frame_allocator::init();

    KERNEL_MEMORY_SPACE.lock().activate();
}
