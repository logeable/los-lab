mod address;
mod frame_allocator;
mod heap;
mod page_table;

pub use frame_allocator::alloc as frame_alloc;
pub use frame_allocator::Frame;
pub use page_table::Flags as PageTableFlags;
pub use page_table::PageTable;
pub use page_table::PageTableEntry;

pub fn init() {
    heap::init();
    frame_allocator::init();
}
