use alloc::vec::Vec;

use core::{
    ffi::{c_char, CStr},
    slice,
};

pub struct AppLoader {
    apps: Vec<AppInfo>,
}

impl AppLoader {
    pub fn load() -> Self {
        extern "C" {
            fn _app_data();
        }

        let number_of_app = unsafe { *(_app_data as *const usize) };

        let mut apps = Vec::new();
        for i in 0..number_of_app {
            let app_info = Self::app_info(i);
            apps.push(app_info);
        }

        AppLoader { apps }
    }

    fn app_info(idx: usize) -> AppInfo {
        extern "C" {
            fn _app_data();
        }

        #[derive(Clone, Copy)]
        struct AppData {
            start: usize,
            end: usize,
            name_ptr: usize,
        }

        let addr = _app_data as usize as *const u64;
        let app_data = unsafe { *(addr.add(1).add(idx * 3) as *const AppData) };

        let name = unsafe { CStr::from_ptr(app_data.name_ptr as *const c_char) }
            .to_str()
            .unwrap();

        let elf_data = unsafe {
            slice::from_raw_parts(app_data.start as *const u8, app_data.end - app_data.start)
        };

        AppInfo { elf_data, name }
    }

    pub fn apps(&self) -> &[AppInfo] {
        &self.apps
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppInfo {
    pub elf_data: &'static [u8],
    pub name: &'static str,
}
