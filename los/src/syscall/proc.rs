use crate::{batch::run_next_app, println};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("app exit_code: {}", exit_code);
    run_next_app()
}
