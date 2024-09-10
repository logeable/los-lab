pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub fn shutdown(failure: bool) -> ! {
    if failure {
        sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::SystemFailure);
    } else {
        sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    }

    unreachable!()
}

pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as u64);
}
