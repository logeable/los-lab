use ansi_rgb::{green, orange, yellow, Foreground};
use core::{
    arch::asm,
    ffi::{c_char, CStr},
    mem, slice,
};
use lazy_static::lazy_static;

use crate::{
    println,
    trap::{self, TrapContext},
};

const APP_BASE_ADDRESS: usize = 0x80400000;
const USER_STACK_SIZE: usize = 4096 * 20;
const KERNEL_STACK_SIZE: usize = 4096 * 20;

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
}

impl AppLoader {
    fn new() -> Self {
        extern "C" {
            fn app_data();
        }

        let app_number = unsafe { *(app_data as *const usize) };

        AppLoader {
            app_number,
            next_app: 0,
        }
    }

    pub fn app_info(&self, idx: usize) -> AppInfo {
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

    pub fn app_number(&self) -> usize {
        self.app_number
    }

    pub fn load_app(&self, idx: usize) {
        let app_info = self.app_info(idx);

        let dest =
            unsafe { slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_info.length) };
        let src = unsafe { slice::from_raw_parts(app_info.start_addr as *mut u8, app_info.length) };

        dest.copy_from_slice(src);

        unsafe { asm!("fence.i") };
    }

    pub fn move_next_app(&mut self) {
        self.next_app += 1;
    }
}

pub fn run_next_app() -> ! {
    {
        let mut loader = APP_LOADER.lock();
        if loader.next_app == loader.app_number {
            println!("{}", "[BATCH] completed".fg(green()));
            loop {
                core::hint::spin_loop();
            }
        }

        loader.load_app(loader.next_app);
        let app_info = loader.app_info(loader.next_app);
        println!(
            "{}",
            format_args!("[BATCH] run {} app: {}", loader.next_app, app_info.name).fg(orange())
        );
        loader.move_next_app();
    }

    let ctx = TrapContext::init(APP_BASE_ADDRESS, USER_STACK.get_sp());
    let ctx_ptr = KERNEL_STACK.push_trap_context(ctx);
    trap::return_to_user(ctx_ptr)
}

#[derive(Debug)]
pub struct AppInfo {
    pub start_addr: usize,
    pub length: usize,
    pub name: &'static str,
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
