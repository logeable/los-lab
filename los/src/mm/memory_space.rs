use super::{
    address::{PhysPageNum, VPNRange, VirtAddr, VirtPageNum},
    frame_allocator::{self, Frame},
    page_table::{Flags, PageTable},
};
use crate::{
    config::{GUARD_PAGE_COUNT, KERNEL_STACK_SIZE, MEMORY_END, USER_STACK_SIZE},
    error,
    mm::address::{PhysAddr, PAGE_SIZE},
    println,
};
use alloc::{collections::btree_map::BTreeMap, format, string::ToString, vec::Vec};
use bitflags::bitflags;
use core::arch::asm;
use elf::endian::AnyEndian;
use riscv::register::satp;

#[derive(Debug)]
struct MapArea {
    vpn_range: VPNRange,
    leaf_frames: BTreeMap<VirtPageNum, Frame>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let vpn_range = VPNRange::new(start_va.floor_vpn(), end_va.ceil_vpn());
        let leaf_frames = BTreeMap::new();

        Self {
            vpn_range,
            leaf_frames,
            map_type,
            map_perm,
        }
    }
    fn map(&mut self, page_table: &mut PageTable) -> error::Result<()> {
        for vpn in self.vpn_range {
            let ppn;
            match self.map_type {
                MapType::Identical => ppn = PhysPageNum::from(vpn.0),
                MapType::Framed => {
                    match frame_allocator::alloc() {
                        Some(frame) => {
                            ppn = frame.ppn;
                            self.leaf_frames.insert(vpn, frame);
                        }
                        None => {
                            return Err(error::KernelError::AllocFrame(
                                "alloc leaf frame failed".to_string(),
                            ));
                        }
                    };
                }
            };

            if let Err(err) = page_table.map(vpn, ppn, self.map_perm.into()) {
                return Err(error::KernelError::PagetableMap(format!(
                    "page table map failed: {:?}",
                    err
                )));
            }
        }

        Ok(())
    }

    fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            match self.map_type {
                MapType::Identical => (),
                MapType::Framed => {
                    self.leaf_frames.remove(&vpn);
                    page_table.unmap(vpn);
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MapPermission:u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

impl From<MapPermission> for Flags {
    fn from(value: MapPermission) -> Self {
        Self::from_bits_truncate(value.bits())
    }
}

#[derive(Debug)]
pub struct MemorySpace {
    l3_page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySpace {
    pub fn new_bare() -> error::Result<Self> {
        match PageTable::new() {
            Ok(l3_page_table) => Ok(Self {
                l3_page_table,
                areas: Vec::new(),
            }),
            Err(err) => Err(error::KernelError::CreatePagetable(format!(
                "create pagetable for bare memory space failed: {:?}",
                err
            ))),
        }
    }

    pub fn new_kernel() -> Self {
        extern "C" {
            fn stext();
            fn etext();
            fn srodata();
            fn erodata();
            fn sdata();
            fn edata();
            fn sbtstack();
            fn ebtstack();
            fn sbss();
            fn ebss();
            fn ekernel();
        }

        let mut mem_space =
            Self::new_bare().expect("create bare memory space for kernel must succeed");

        mem_space
            .add_trampoline_area()
            .expect("add trampoline area must succeed");

        mem_space
            .add_identical_area(
                (stext as usize).into(),
                (etext as usize).into(),
                MapPermission::R | MapPermission::X,
            )
            .expect("add .text area must succeed");

        mem_space
            .add_identical_area(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapPermission::R,
            )
            .expect("add .rodata area must succeed");

        mem_space
            .add_identical_area(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapPermission::R | MapPermission::W,
            )
            .expect("add .data area must succeed");

        mem_space
            .add_identical_area(
                (sbtstack as usize).into(),
                (ebtstack as usize).into(),
                MapPermission::R | MapPermission::W,
            )
            .expect("add .bss area must succeed");

        mem_space
            .add_identical_area(
                (sbss as usize).into(),
                (ebss as usize).into(),
                MapPermission::R | MapPermission::W,
            )
            .expect("add .bss area must succeed");

        mem_space
            .add_identical_area(
                (ekernel as usize).into(),
                (MEMORY_END as usize).into(),
                MapPermission::R | MapPermission::W,
            )
            .expect("add memory area must succeed");

        mem_space
    }

    pub fn new_elf(elf_data: &[u8]) -> error::Result<(Self, usize, usize)> {
        let mut mem_space = Self::new_bare().map_err(|e| {
            error::KernelError::CreateMemorySpace(format!("create memory space failed: {:?}", e))
        })?;

        let mut max_vpn = VirtPageNum(0);

        let file = elf::ElfBytes::<AnyEndian>::minimal_parse(elf_data)
            .map_err(|e| error::KernelError::ParseELF(format!("parse elf failed: {:?}", e)))?;
        for segment in file
            .segments()
            .ok_or(error::KernelError::ELFProgramHeader("".to_string()))?
            .iter()
            .filter(|v| v.p_type == elf::abi::PT_LOAD)
        {
            let data = file.segment_data(&segment).map_err(|e| {
                error::KernelError::ELFSegmentData(format!("read segment data failed: {:?}", e))
            })?;
            let start_va = VirtAddr(segment.p_vaddr as usize);
            let end_va = start_va + segment.p_memsz as usize;
            let mut map_perm = MapPermission::U;
            if segment.p_flags & elf::abi::PF_R != 0 {
                map_perm |= MapPermission::R;
            }
            if segment.p_flags & elf::abi::PF_W != 0 {
                map_perm |= MapPermission::W;
            }
            if segment.p_flags & elf::abi::PF_X != 0 {
                map_perm |= MapPermission::X;
            }
            let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
            let end_vpn = map_area.vpn_range.end();
            if end_vpn > max_vpn {
                max_vpn = end_vpn;
            }
            mem_space
                .add_map_area_with_data(map_area, data)
                .map_err(|e| {
                    error::KernelError::AddMapArea(format!(
                        "add elf segment (va: {:#x}) failed: {:?}",
                        segment.p_vaddr, e
                    ))
                })?;
        }

        mem_space.add_trampoline_area().map_err(|e| {
            error::KernelError::AddMapArea(format!("add trampoline map area failed: {:?}", e))
        })?;

        let trap_context_start_va = trap_context_va();
        mem_space
            .add_framed_area(
                trap_context_start_va,
                trap_context_start_va + PAGE_SIZE,
                MapPermission::R | MapPermission::W,
            )
            .map_err(|e| {
                error::KernelError::AddMapArea(format!("add trap context map area failed: {:?}", e))
            })?;

        let user_stack_start_va = VirtPageNum(max_vpn.0 + GUARD_PAGE_COUNT).into();
        let user_stack_end_va = user_stack_start_va + USER_STACK_SIZE;
        mem_space
            .add_framed_area(
                user_stack_start_va,
                user_stack_end_va,
                MapPermission::U | MapPermission::R | MapPermission::W,
            )
            .map_err(|e| {
                error::KernelError::AddMapArea(format!("add user stack map area failed: {:?}", e))
            })?;
        Ok((
            mem_space,
            user_stack_end_va.into(),
            file.ehdr.e_entry as usize,
        ))
    }

    fn add_trampoline_area(&mut self) -> error::Result<()> {
        extern "C" {
            fn strampoline();
            fn etrampoline();
        }

        assert!(strampoline as usize % PAGE_SIZE == 0);
        assert_eq!(etrampoline as usize - strampoline as usize, PAGE_SIZE);

        let ppn = PhysAddr::from(strampoline as usize).floor_ppn();
        let vpn = trampoline_va().floor_vpn();

        self.l3_page_table
            .map(vpn, ppn, (MapPermission::R | MapPermission::X).into())?;

        Ok(())
    }

    pub fn add_app_kernel_stack_area(&mut self, app_id: usize) -> error::Result<usize> {
        let end_va =
            kernel_stack_top_va() - app_id * (KERNEL_STACK_SIZE + GUARD_PAGE_COUNT * PAGE_SIZE);
        let start_va = end_va - KERNEL_STACK_SIZE;
        self.add_framed_area(start_va, end_va, MapPermission::R | MapPermission::W)?;

        Ok(end_va.into())
    }

    fn add_map_area(&mut self, mut map_area: MapArea) -> error::Result<()> {
        if let Err(err) = map_area.map(&mut self.l3_page_table) {
            return Err(error::KernelError::MapArea(format!(
                "map failed: {:?}",
                err
            )));
        }
        self.areas.push(map_area);

        Ok(())
    }

    fn add_map_area_with_data(&mut self, map_area: MapArea, data: &[u8]) -> error::Result<()> {
        assert_eq!(map_area.map_type, MapType::Framed);
        assert!(map_area.vpn_range.memory_size() >= data.len());

        let vpn_range = map_area.vpn_range;
        self.add_map_area(map_area)?;

        data.chunks(PAGE_SIZE)
            .zip(vpn_range)
            .for_each(|(page_data, vpn)| {
                let ppn = self.l3_page_table.translate(vpn).unwrap().ppn();
                let page_data_dest = unsafe { ppn.get_bytes_array_mut() };
                page_data_dest[..page_data.len()].copy_from_slice(page_data);
            });

        Ok(())
    }

    pub fn add_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_perm: MapPermission,
    ) -> error::Result<()> {
        self.add_map_area(MapArea::new(start_va, end_va, MapType::Framed, map_perm))
    }

    pub fn add_identical_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_perm: MapPermission,
    ) -> error::Result<()> {
        self.add_map_area(MapArea::new(start_va, end_va, MapType::Identical, map_perm))
    }

    pub fn activate(&self) {
        let satp = self.l3_page_table.satp();
        satp::write(satp);
        unsafe {
            asm!("sfence.vma");
        }
    }

    pub fn page_table(&self) -> &PageTable {
        &self.l3_page_table
    }
}

pub fn trampoline_va() -> VirtAddr {
    VirtAddr::HIGH_HALF_MAX - PAGE_SIZE + 1
}

fn kernel_stack_top_va() -> VirtAddr {
    trampoline_va()
}

pub fn trap_context_va() -> VirtAddr {
    trampoline_va() - PAGE_SIZE
}
