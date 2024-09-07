use crate::println;

use super::manager::MAX_APPS;
use core::{
    ffi::{c_char, CStr},
    slice,
};

pub struct AppLoader {
    number_of_app: usize,
    apps: [Option<AppInfo>; MAX_APPS],
}

impl AppLoader {
    pub fn new() -> Self {
        extern "C" {
            fn _app_data();
        }

        let number_of_app = unsafe { *(_app_data as *const usize) };

        let mut apps = [None; MAX_APPS];
        for i in 0..number_of_app {
            let app_info = Self::app_info(i);
            apps[i] = Some(app_info);
            Self::load_app(&app_info);
        }

        AppLoader {
            number_of_app,
            apps,
        }
    }

    pub fn get_app_info(&self, idx: usize) -> Option<&AppInfo> {
        self.apps.get(idx).and_then(|v| v.as_ref())
    }

    pub fn get_number_of_app(&self) -> usize {
        self.number_of_app
    }

    fn load_app(app_info: &AppInfo) {
        let dest = unsafe { slice::from_raw_parts_mut(app_info.entry as *mut u8, app_info.length) };
        let src = unsafe { slice::from_raw_parts(app_info.start_addr as *mut u8, app_info.length) };

        dest.copy_from_slice(src);
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
            entry: usize,
        }

        let addr = _app_data as usize as *const u64;
        let app_data = unsafe { *(addr.add(1).add(idx * 4) as *const AppData) };

        let name = unsafe { CStr::from_ptr(app_data.name_ptr as *const c_char) }
            .to_str()
            .unwrap();

        AppInfo {
            start_addr: app_data.start,
            length: app_data.end - app_data.start,
            name,
            entry: app_data.entry,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppInfo {
    start_addr: usize,
    length: usize,
    pub name: &'static str,
    pub entry: usize,
}
