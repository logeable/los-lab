mod loader;
pub mod manager;
pub mod pid;
pub mod processor;
mod tcb;

use crate::println;

pub fn init() {
    manager::create_init_proc_and_push_to_runq().expect("create init proc must succeed");
}

pub(crate) fn print_apps() {
    for (i, name) in manager::list_apps().iter().enumerate() {
        println!("{}: {}", i, name);
    }
}
