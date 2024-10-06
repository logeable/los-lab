use core::{mem, ops};

use super::page_table::PageTableEntry;

const PAGE_OFFSET_WIDTH: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_OFFSET_WIDTH;
const PA_WIDTH_SV39: usize = 56;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_OFFSET_WIDTH;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PhysAddr(pub usize);

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

impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_OFFSET_WIDTH)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct VirtAddr(pub usize);

impl VirtAddr {
    pub const HIGH_HALF_MAX: Self = Self(u64::MAX as usize);
    pub const LOW_HALF_MAX: Self = Self((1 << 38) - 1);

    pub fn floor_vpn(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil_vpn(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }

    pub fn is_page_aligned(&self) -> bool {
        self.0 % PAGE_SIZE == 0
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(value: VirtPageNum) -> Self {
        Self(value.0 << PAGE_OFFSET_WIDTH)
    }
}

impl ops::Sub<usize> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl ops::Add<usize> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        VirtAddr(value)
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        value.0
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PhysPageNum(pub usize);

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
        let start = PhysAddr::from(*self).0;
        unsafe { &mut *(start as *mut T) }
    }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct VirtPageNum(pub usize);

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

    pub fn offset(&self, offset: usize) -> Self {
        Self(self.0 + offset)
    }
}

impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VPNRange {
    start: VirtPageNum,
    end: VirtPageNum,
}

impl VPNRange {
    pub fn new(start: VirtPageNum, end: VirtPageNum) -> Self {
        Self { start, end }
    }

    pub fn memory_size(&self) -> usize {
        self.into_iter().count() * PAGE_SIZE
    }

    #[allow(dead_code)]
    pub fn start(&self) -> VirtPageNum {
        self.start
    }

    pub fn end(&self) -> VirtPageNum {
        self.end
    }
}

impl IntoIterator for VPNRange {
    type Item = VirtPageNum;

    type IntoIter = VPNRangeIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            current: self.start,
            end: self.end,
        }
    }
}

pub struct VPNRangeIter {
    current: VirtPageNum,
    end: VirtPageNum,
}

impl Iterator for VPNRangeIter {
    type Item = VirtPageNum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let current = self.current;
            self.current = VirtPageNum::from(self.current.0 + 1);

            Some(current)
        } else {
            None
        }
    }
}
