use core::ops::Range;

use alloc::format;
use dtb_walker::{Dtb, HeaderError, Property, WalkOperation};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref DEVICE_INFO: Mutex<DeviceInfo> = Mutex::new(DeviceInfo::default());
}

#[derive(Debug, Clone, Default)]
pub struct DeviceInfo {
    pub memory: Range<usize>,
    pub cpu_time_base_freq: usize,
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
            if !name.starts_with("memory") && !name.starts_with("cpu") {
                return WalkOperation::StepOver;
            }
            WalkOperation::StepInto
        }
        dtb_walker::DtbObj::Property(mut property) => {
            let name = core::str::from_utf8(path.last()).unwrap();
            if name.starts_with("memory") {
                if let Property::Reg(reg) = &mut property {
                    let mut info = DEVICE_INFO.lock();
                    info.memory = reg.next().unwrap();
                }
            }

            if name.starts_with("cpus") {
                if let Property::General { name, value } = &property {
                    let name = name.as_str().unwrap();
                    if name == "timebase-frequency" {
                        let mut bytes = [0u8; 4];
                        bytes[0] = value[0];
                        bytes[1] = value[1];
                        bytes[2] = value[2];
                        bytes[3] = value[3];

                        let mut info = DEVICE_INFO.lock();
                        info.cpu_time_base_freq = u32::from_be_bytes(bytes) as usize;
                    }
                }
            }

            WalkOperation::StepOver
        }
    });
}

pub fn get_device_info() -> DeviceInfo {
    DEVICE_INFO.lock().clone()
}
