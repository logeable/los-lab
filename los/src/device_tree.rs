use core::ops::Range;

use crate::println;
use alloc::format;
use dtb_walker::{utils::indent, Dtb, HeaderError, Property, WalkOperation};
use lazy_static::lazy_static;
use spin::Mutex;

const INDENT_WIDTH: usize = 4;

lazy_static! {
    static ref DEVICE_INFO: Mutex<DeviceInfo> = Mutex::new(DeviceInfo {
        memory: Range::default()
    });
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub memory: Range<usize>,
}

pub fn init(device_tree_pa: usize) {
    let dtb = unsafe {
        Dtb::from_raw_parts_filtered(device_tree_pa as _, |e| {
            matches!(
                e,
                HeaderError::Misaligned(4) | HeaderError::LastCompVersion(16)
            )
        })
    }
    .map_err(|e| format!("verify header failed: {e:?}"))
    .unwrap();

    dtb.walk(|path, obj| match obj {
        dtb_walker::DtbObj::SubNode { name } => {
            let name = core::str::from_utf8(name).unwrap();
            if !name.starts_with("memory") {
                return WalkOperation::StepOver;
            }
            // println!("{}{}/{}", indent(path.level(), INDENT_WIDTH), path, name);
            WalkOperation::StepInto
        }
        dtb_walker::DtbObj::Property(property) => {
            let name = core::str::from_utf8(path.last()).unwrap();

            // println!("{}{:?}", indent(path.level(), INDENT_WIDTH), property);

            if let Property::Reg(mut reg) = property {
                if name.starts_with("memory") {
                    let mut info = DEVICE_INFO.lock();
                    info.memory = reg.next().unwrap();
                }
            }

            WalkOperation::StepOver
        }
    });
}

pub fn get_device_info() -> DeviceInfo {
    DEVICE_INFO.lock().clone()
}
