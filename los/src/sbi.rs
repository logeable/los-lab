use sbi_rt::Physical;

pub fn console_read_bytes(buf: &mut [u8]) -> isize {
    let bytes = Physical::new(buf.len(), buf.as_ptr() as usize, 0);
    let ret = sbi_rt::console_read(bytes);

    if ret.is_err() {
        return -1;
    }

    ret.value as isize
}

pub fn console_write_byte(c: usize) {
    sbi_rt::console_write_byte(c as u8);
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
