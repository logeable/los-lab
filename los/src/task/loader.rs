use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use core::{
    ffi::{c_char, CStr},
    slice,
};

pub struct AppLoader {
    apps: BTreeMap<String, AppInfo>,
}

impl AppLoader {
    pub fn load() -> Self {
        extern "C" {
            fn _app_data();
        }

        let number_of_app = unsafe { *(_app_data as *const usize) };

        let mut apps = BTreeMap::new();
        for i in 0..number_of_app {
            let app_info = Self::load_app_info(i);
            apps.insert(app_info.name.to_string(), app_info);
        }

        AppLoader { apps }
    }

    fn load_app_info(idx: usize) -> AppInfo {
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

        AppInfo {
            name,
            elf_start: app_data.start,
            elf_size: app_data.end - app_data.start,
        }
    }

    pub fn load_app_elf(&self, name: &str) -> Option<&'static [u8]> {
        let app_info = self.apps.get(name)?;

        let elf_data =
            unsafe { slice::from_raw_parts(app_info.elf_start as *const u8, app_info.elf_size) };

        Some(elf_data)
    }

    pub fn app_names(&self) -> Vec<String> {
        self.apps.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppInfo {
    pub name: &'static str,
    pub elf_start: usize,
    pub elf_size: usize,
}
