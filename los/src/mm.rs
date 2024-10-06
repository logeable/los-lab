use crate::device_tree;
use crate::error;
use lazy_static::lazy_static;
use spin::Mutex;

pub mod address;
mod frame_allocator;
mod heap;
mod memory_space;
mod page_table;

#[allow(unused_imports)]
pub use heap::kernel_heap_stats;
pub use memory_space::trampoline_va;
pub use memory_space::trap_context_va;
pub use memory_space::MemorySpace;
pub use page_table::PageTable;

lazy_static! {
    pub static ref KERNEL_MEMORY_SPACE: Mutex<memory_space::MemorySpace> = {
        let mem_range = &device_tree::get_device_info().memory;
        Mutex::new(memory_space::MemorySpace::new_kernel(mem_range))
    };
}

pub fn init(device_info: &device_tree::DeviceInfo) {
    heap::init(&device_info.memory);
    frame_allocator::init(&device_info.memory);

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
