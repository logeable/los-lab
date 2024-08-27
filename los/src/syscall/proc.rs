use ansi_rgb::{orange, white, Foreground};

use crate::{batch::run_next_app, println};

pub fn sys_exit(exit_code: i32) -> ! {
    println!(
        "{}",
        format_args!("app exit_code: {}", exit_code).fg(white())
    );
    run_next_app()
}
