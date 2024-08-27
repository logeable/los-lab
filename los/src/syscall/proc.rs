use ansi_rgb::{orange, white, Foreground};

use crate::{
    batch::{run_next_app, APP_LOADER},
    println,
};

pub fn sys_exit(exit_code: i32) -> ! {
    {
        let mut loader = APP_LOADER.lock();
        loader.update_end_time();
        println!(
            "{}",
            format_args!(
                "app exit_code: {}, dur: {}",
                exit_code,
                loader.get_app_duration()
            )
            .fg(white())
        );
    }
    run_next_app()
}
