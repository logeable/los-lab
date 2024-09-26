use core::{any::Any, arch::asm};

use alloc::{collections::btree_map::BTreeMap, format, string::ToString, vec::Vec};
use bitflags::bitflags;
use elf::endian::AnyEndian;
use riscv::register::satp;

use crate::{config::MEMORY_END, error, mm::address::PAGE_SIZE, println};

use super::{
    address::{PhysPageNum, VPNRange, VirtAddr, VirtPageNum},
    frame_allocator::{self, Frame},
    page_table::{Flags, PageTable},
};

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
                                "alloc frame failed to map area".to_string(),
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
    #[derive(Clone, Copy)]
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
            Err(err) => Err(error::KernelError::CreateMemorySpace(format!(
                "new bare memory space failed: {:?}",
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

    pub fn new_elf(elf_data: &[u8]) -> error::Result<Self> {
        let file = elf::ElfBytes::<AnyEndian>::minimal_parse(elf_data).unwrap();

        file.segments()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(i, s)| println!("{:04}: {:?}", i, s));

        todo!()
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
}
