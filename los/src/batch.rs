use ansi_rgb::{green, orange, Foreground};
use core::{
    ffi::{c_char, CStr},
    mem, slice,
};
use lazy_static::lazy_static;
use riscv::register::time;

use crate::{
    println,
    trap::{self, TrapContext},
};

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APPS: usize = 100;

lazy_static! {
    pub static ref APP_LOADER: spin::Mutex<AppLoader> = {
        let loader = AppLoader::new();

        spin::Mutex::new(loader)
    };
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

pub struct AppLoader {
    app_number: usize,
    next_app: usize,
    start_time: usize,
    end_time: usize,
    apps: [Option<AppInfo>; MAX_APPS],
}

impl AppLoader {
    fn new() -> Self {
        extern "C" {
            fn app_data();
        }

        let app_number = unsafe { *(app_data as *const usize) };

        let mut apps = [None; MAX_APPS];
        for i in 0..app_number {
            let app_info = Self::app_info(i);
            apps[i] = Some(app_info);
            Self::load_app(&app_info);
        }

        AppLoader {
            app_number,
            next_app: 0,
            start_time: 0,
            end_time: 0,
            apps,
        }
    }

    pub fn get_app_info(&self, idx: usize) -> Option<&AppInfo> {
        self.apps.get(idx).and_then(|v| v.as_ref())
    }

    fn app_info(idx: usize) -> AppInfo {
        extern "C" {
            fn app_data();
        }

        #[derive(Clone, Copy)]
        struct AppData {
            start: usize,
            end: usize,
            name_ptr: usize,
            entry_ptr: usize,
        }

        let addr = app_data as usize as *const u64;
        let app_data = unsafe { *(addr.add(1).add(idx * 4) as *const AppData) };

        let name = unsafe { CStr::from_ptr(app_data.name_ptr as *const c_char) }
            .to_str()
            .unwrap();

        let entry = unsafe { *(app_data.entry_ptr as *const u64) } as usize;

        AppInfo {
            start_addr: app_data.start,
            length: app_data.end - app_data.start,
            name,
            entry,
        }
    }

    pub fn app_number(&self) -> usize {
        self.app_number
    }

    fn load_app(app_info: &AppInfo) {
        let dest = unsafe { slice::from_raw_parts_mut(app_info.entry as *mut u8, app_info.length) };
        let src = unsafe { slice::from_raw_parts(app_info.start_addr as *mut u8, app_info.length) };

        dest.copy_from_slice(src);
    }

    pub fn move_next_app(&mut self) {
        self.next_app += 1;
    }

    pub fn update_start_time(&mut self) {
        self.start_time = time::read();
    }

    pub fn update_end_time(&mut self) {
        self.end_time = time::read();
    }

    pub fn get_app_duration(&self) -> usize {
        return self.end_time - self.start_time;
    }
}

pub fn run_next_app() -> ! {
    let ctx_ptr = {
        let mut loader = APP_LOADER.lock();

        loader.update_start_time();
        match loader.get_app_info(loader.next_app) {
            Some(app_info) => {
                println!(
                    "{}",
                    format_args!(
                        "[BATCH] run {} app: {}({:#x})",
                        loader.next_app, app_info.name, app_info.entry
                    )
                    .fg(orange())
                );

                let ctx = TrapContext::init(app_info.entry, USER_STACK.get_sp());
                loader.move_next_app();
                KERNEL_STACK.push_trap_context(ctx)
            }
            None => {
                println!("{}", "[BATCH] completed".fg(green()));
                loop {
                    core::hint::spin_loop();
                }
            }
        }
    };

    trap::return_to_user(ctx_ptr)
}

#[derive(Debug, Clone, Copy)]
pub struct AppInfo {
    pub start_addr: usize,
    pub length: usize,
    pub name: &'static str,
    pub entry: usize,
}

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    fn push_trap_context(&self, ctx: TrapContext) -> *mut TrapContext {
        let sp = self.get_sp() - mem::size_of::<TrapContext>();
        let ctx_ptr = sp as *mut TrapContext;
        unsafe { ctx_ptr.write(ctx) };
        ctx_ptr
    }
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}
