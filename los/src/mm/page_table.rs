use core::fmt::Debug;

use alloc::vec::Vec;
use alloc::{string::ToString, vec};
use bitflags::bitflags;

use crate::error;

use super::{
    address::{PhysPageNum, VirtPageNum},
    frame_allocator::{self, Frame},
};

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Flags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: Flags) -> Self {
        let bits = ppn.0 << 10 | flags.bits() as usize;

        Self { bits }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10).into()
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits_retain(self.bits as u8)
    }

    pub fn is_valid(&self) -> bool {
        self.flags().intersects(Flags::V)
    }

    pub fn is_writable(&self) -> bool {
        self.flags().intersects(Flags::W)
    }

    pub fn is_readable(&self) -> bool {
        self.flags().intersects(Flags::R)
    }

    pub fn is_executable(&self) -> bool {
        self.flags().intersects(Flags::X)
    }

    pub fn is_user(&self) -> bool {
        self.flags().intersects(Flags::U)
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("bits", &self.bits)
            .field("ppn", &self.ppn())
            .field("flags", &self.flags())
            .finish()
    }
}

#[derive(Debug)]
pub struct PageTable {
    root_ppn: PhysPageNum,
    dir_frames: Vec<Frame>,
}

impl PageTable {
    pub fn new() -> error::Result<Self> {
        match frame_allocator::alloc() {
            Some(frame) => Ok(Self {
                root_ppn: frame.ppn,
                dir_frames: vec![frame],
            }),
            None => Err(error::KernelError::AllocFrame(
                "new root page table failed".to_string(),
            )),
        }
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: Flags) -> error::Result<()> {
        let l3_ppn = self.root_ppn;
        let l3_ptes = unsafe { l3_ppn.get_pte_array_mut() };
        let l3_index = vpn.get_level_3_index();
        let pte = &mut l3_ptes[l3_index];
        if !pte.is_valid() {
            match frame_allocator::alloc() {
                Some(frame) => {
                    *pte = PageTableEntry::new(frame.ppn, Flags::V);
                    self.dir_frames.push(frame);
                }
                None => {
                    return Err(error::KernelError::AllocFrame(
                        "allocate level2 page table frame failed".to_string(),
                    ))
                }
            }
        }

        let l2_ppn = pte.ppn();
        let l2_ptes = unsafe { l2_ppn.get_pte_array_mut() };
        let l2_index = vpn.get_level_2_index();
        let pte = &mut l2_ptes[l2_index];
        if !pte.is_valid() {
            match frame_allocator::alloc() {
                Some(frame) => {
                    *pte = PageTableEntry::new(frame.ppn, Flags::V);
                    self.dir_frames.push(frame);
                }
                None => {
                    return Err(error::KernelError::AllocFrame(
                        "allocate level1 page table frame failed".to_string(),
                    ))
                }
            }
        }

        let l1_ppn = pte.ppn();
        let l1_ptes = unsafe { l1_ppn.get_pte_array_mut() };
        let l1_index = vpn.get_level_1_index();
        let pte = &mut l1_ptes[l1_index];
        if !pte.is_valid() {
            *pte = PageTableEntry::new(ppn, flags | Flags::V);
        }

        Ok(())
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        match self.find_pte_mut(vpn) {
            Some(pte) => {
                let pte = unsafe { &mut *pte };
                *pte = PageTableEntry::empty();
            }
            None => panic!("unmap a none page {:?}", vpn),
        }
    }

    fn find_pte_mut(&self, vpn: VirtPageNum) -> Option<*mut PageTableEntry> {
        let ppn = self.root_ppn;
        let l3_ptes = unsafe { ppn.get_pte_array_mut() };
        let l3_index = vpn.get_level_3_index();
        let pte = l3_ptes[l3_index];
        if !pte.is_valid() {
            return None;
        }

        let ppn = pte.ppn();
        let l2_ptes = unsafe { ppn.get_pte_array_mut() };
        let l2_index = vpn.get_level_2_index();
        let pte = l2_ptes[l2_index];
        if !pte.is_valid() {
            return None;
        }

        let ppn = pte.ppn();
        let l1_ptes = unsafe { ppn.get_pte_array_mut() };
        let l1_index = vpn.get_level_1_index();
        let pte = &mut l1_ptes[l1_index];
        if !pte.is_valid() {
            return None;
        }

        Some(pte)
    }

    pub fn satp(&self) -> usize {
        (8 << 60) | (0 << 44) | self.root_ppn.0
    }

    pub fn from_satp(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp),
            dir_frames: Vec::new(),
        }
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte_mut(vpn).map(|pte| unsafe { *pte })
    }

    pub fn root_ppn(&self) -> PhysPageNum {
        self.root_ppn
    }
}
