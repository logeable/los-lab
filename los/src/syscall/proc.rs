use ansi_rgb::{green, Foreground};

use crate::{println, task};

pub fn sys_exit(exit_code: i32) -> ! {
    {
        println!(
            "{}",
            format_args!("app exit_code: {}", exit_code,).fg(green())
        );
    }

    task::exit_current_task_and_schedule()
}
