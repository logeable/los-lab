use core::mem;

use super::page_table::PageTableEntry;

const PAGE_OFFSET_WIDTH: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_OFFSET_WIDTH;
const VA_WIDTH_SV39: usize = 39;
const PA_WIDTH_SV39: usize = 56;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_OFFSET_WIDTH;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_OFFSET_WIDTH;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PhysAddr(pub usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct VirtAddr(pub usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PhysPageNum(pub usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct VirtPageNum(pub usize);

impl PhysAddr {
    pub fn floor_ppn(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil_ppn(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
}

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_OFFSET_WIDTH)
    }
}

impl PhysPageNum {
    pub fn clear(&mut self) {
        unsafe {
            self.get_bytes_array_mut().fill(0);
        }
    }

    pub unsafe fn get_bytes_array_mut(&self) -> &mut [u8] {
        let start = PhysAddr::from(*self).0;
        let end = start + PAGE_SIZE;
        unsafe { core::slice::from_raw_parts_mut(start as *mut u8, end - start) }
    }

    pub unsafe fn get_pte_array_mut(&self) -> &mut [PageTableEntry] {
        let start = PhysAddr::from(*self).0;
        let len = PAGE_SIZE / mem::size_of::<PageTableEntry>();
        unsafe { core::slice::from_raw_parts_mut(start as *mut PageTableEntry, len) }
    }

    pub unsafe fn get_mut<T>(&self) -> &mut T {
        unsafe { &mut *(self.0 as *mut T) }
    }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl VirtPageNum {
    pub fn get_level_1_index(&self) -> usize {
        self.0 & 0x1ff
    }

    pub fn get_level_2_index(&self) -> usize {
        (self.0 >> 9) & 0x1ff
    }

    pub fn get_level_3_index(&self) -> usize {
        (self.0 >> 18) & 0x1ff
    }
}

impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self {
        Self(value & ((1 << VPN_WIDTH_SV39) - 1))
    }
}
