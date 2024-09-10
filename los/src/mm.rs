use buddy_system_allocator::LockedHeap;

const HEAP_SIZE: usize = 1 << 26;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

static HEAP_SPACE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, HEAP_SIZE)
    };
}
