use core::{
    ffi::{c_char, CStr},
    mem,
};

pub struct AppLoader {
    app_number: u64,
}

impl AppLoader {
    pub fn new() -> Self {
        extern "C" {
            fn app_data();
        }

        let app_number = unsafe { *(app_data as *const u64) };

        AppLoader { app_number }
    }

    pub fn load_app(&self, idx: usize) -> AppInfo {
        extern "C" {
            fn app_data();
        }

        let app_info_addr = unsafe { (app_data as *const u64).add(1 + idx * 3) };
        let start = unsafe { *app_info_addr } as usize;
        let end_addr = unsafe { app_info_addr.add(1) };
        let end = unsafe { *end_addr } as usize;
        let name_addr = unsafe { *end_addr.add(1) };

        let name = unsafe { CStr::from_ptr(name_addr as *const c_char) };

        AppInfo {
            start_addr: start,
            length: end - start,
            name: name.to_str().unwrap_or("unknown"),
        }
    }

    pub fn app_number(&self) -> u64 {
        self.app_number
    }
}

#[derive(Debug)]
pub struct AppInfo {
    pub start_addr: usize,
    pub length: usize,
    pub name: &'static str,
}
