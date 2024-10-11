use core::fmt::Debug;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{string::ToString, vec};
use bitflags::bitflags;

use crate::error;

use super::address::{PhysAddr, VirtAddr};
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

    pub fn translate_vpn(&self, vpn: VirtPageNum) -> error::Result<PageTableEntry> {
        self.find_pte_mut(vpn)
            .map(|pte| unsafe { *pte })
            .ok_or(error::KernelError::PteNotFound(format!(
                "find pte by vpn failed, vpn: {:?}",
                vpn
            )))
    }

    pub fn translate_va(&self, va: VirtAddr) -> error::Result<PhysAddr> {
        let pte = self.translate_vpn(va.floor_vpn())?;

        let pa = PhysAddr::from(pte.ppn());
        Ok(pa.offset(va.page_offset()))
    }

    pub fn translate_c_str(&self, ptr: VirtAddr) -> error::Result<String> {
        let mut s = String::new();

        let mut pa = self.translate_va(ptr)?;
        loop {
            let c = unsafe { *(pa.0 as *const u8) };
            if c == 0 {
                break;
            }
            s.push(c as char);
            pa = pa.offset(1)
        }

        Ok(s)
    }

    pub fn translate_bytes(&self, ptr: VirtAddr, len: usize) -> error::Result<Vec<&mut [u8]>> {
        let end_va = ptr + len;

        let mut result: Vec<&'static mut [u8]> = Vec::new();

        let mut addr = ptr;
        while addr < end_va {
            let addr_va = VirtAddr::from(addr);

            let page_vpn = addr_va.floor_vpn();
            let page_ppn = self
                .translate_vpn(page_vpn)
                .map_err(|e| {
                    error::KernelError::Translate(format!("translate start vpn failed: {:?}", e))
                })?
                .ppn();

            let page_start_va = VirtAddr::from(page_vpn);
            let page_end_va = VirtAddr::from(page_vpn.offset(1));

            let chunk_end_va = page_end_va.0.min(end_va.0);
            let chunk_start_va = addr_va.0;
            let chunk_page_offset = addr_va.0 - page_start_va.0;
            let chunk_len = chunk_end_va - chunk_start_va;

            let page_start_pa = PhysAddr::from(page_ppn);
            let chunk_start_pa = page_start_pa.0 + chunk_page_offset;
            let chunk =
                unsafe { core::slice::from_raw_parts_mut(chunk_start_pa as *mut u8, chunk_len) };

            addr = addr + chunk_len;

            result.push(chunk);
        }

        Ok(result)
    }

    fn fork_dir_pte_frames(
        &mut self,
        src_ppns: &mut [PhysPageNum],
        dst_ppns: &mut [PhysPageNum],
    ) -> error::Result<(Vec<PhysPageNum>, Vec<PhysPageNum>)> {
        let mut next_level_src_ppns = Vec::new();
        let mut next_level_dst_ppns = Vec::new();

        for (src_ppn, dst_ppn) in src_ppns.iter().zip(dst_ppns) {
            let src_ptes = unsafe { src_ppn.get_pte_array_mut() };
            let dst_ptes = unsafe { dst_ppn.get_pte_array_mut() };

            for (src_pte, dst_pte) in src_ptes.iter_mut().zip(dst_ptes) {
                if src_pte.is_valid() {
                    match frame_allocator::alloc() {
                        Some(frame) => {
                            next_level_src_ppns.push(src_pte.ppn());
                            next_level_dst_ppns.push(frame.ppn);

                            *dst_pte = PageTableEntry::new(frame.ppn, src_pte.flags());
                            self.dir_frames.push(frame);
                        }
                        None => {
                            return Err(error::KernelError::AllocFrame(
                                "allocate frame failed".to_string(),
                            ))
                        }
                    }
                }
            }
        }

        Ok((next_level_src_ppns, next_level_dst_ppns))
    }

    fn fork_data_pte_frames(
        &mut self,
        src_ppns: &mut [PhysPageNum],
        dst_ppns: &mut [PhysPageNum],
    ) -> error::Result<()> {
        for (src_ppn, dst_ppn) in src_ppns.iter().zip(dst_ppns) {
            let src_ptes = unsafe { src_ppn.get_pte_array_mut() };
            let dst_ptes = unsafe { dst_ppn.get_pte_array_mut() };

            for (src_pte, dst_pte) in src_ptes.iter_mut().zip(dst_ptes) {
                if src_pte.is_valid() {
                    match frame_allocator::alloc() {
                        Some(frame) => {
                            let src_ppn = src_pte.ppn();

                            let dst = unsafe { frame.ppn.get_bytes_array_mut() };
                            let src = unsafe { src_ppn.get_bytes_array_mut() };
                            dst.copy_from_slice(src);

                            *dst_pte = PageTableEntry::new(frame.ppn, src_pte.flags());
                            self.dir_frames.push(frame);
                        }
                        None => {
                            return Err(error::KernelError::AllocFrame(
                                "allocate frame failed".to_string(),
                            ))
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn fork_from_page_table(&mut self, src: &Self) -> error::Result<()> {
        let mut src_l3_ppns = vec![src.root_ppn];
        let mut dst_l3_ppns = vec![self.root_ppn];

        let (mut src_l2_ppns, mut dst_l2_ppns) =
            self.fork_dir_pte_frames(&mut src_l3_ppns, &mut dst_l3_ppns)?;

        let (mut src_l1_ppns, mut dst_l1_ppns) =
            self.fork_dir_pte_frames(&mut src_l2_ppns, &mut dst_l2_ppns)?;

        self.fork_data_pte_frames(&mut src_l1_ppns, &mut dst_l1_ppns)?;

        Ok(())
    }
}
