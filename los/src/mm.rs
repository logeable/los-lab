use crate::error;
use lazy_static::lazy_static;
use spin::Mutex;

pub mod address;
mod frame_allocator;
mod heap;
mod memory_space;
mod page_table;

pub use memory_space::trampoline_va;
pub use memory_space::trap_context_va;
pub use memory_space::MemorySpace;

lazy_static! {
    pub static ref KERNEL_MEMORY_SPACE: Mutex<memory_space::MemorySpace> =
        Mutex::new(memory_space::MemorySpace::new_kernel());
}

pub fn init() {
    heap::init();
    frame_allocator::init();

    KERNEL_MEMORY_SPACE.lock().activate();
}

pub fn kernel_satp() -> usize {
    KERNEL_MEMORY_SPACE.lock().page_table().satp()
}

pub fn build_app_mem_space(
    elf_data: &[u8],
) -> error::Result<(memory_space::MemorySpace, usize, usize)> {
    memory_space::MemorySpace::new_elf(elf_data)
}

pub fn add_app_kernel_stack_area_in_kernel_space(app_id: usize) -> error::Result<usize> {
    KERNEL_MEMORY_SPACE.lock().add_app_kernel_stack_area(app_id)
}
