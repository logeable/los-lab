#![no_std]
#![no_main]
use user::{entry, exec, fork, println, wait};

entry!(main);

fn main() -> i32 {
    let shell = "lshell";
    match fork() {
        Ok(fork_proc) => match fork_proc {
            user::ForkProc::Child => {
                exec(shell).expect("exec shell must succeed");
            }
            user::ForkProc::Parent(pid) => loop {
                println!("subprocess {}({}) started", shell, pid);
                loop {
                    let wr = wait().expect("wait must succeed");
                    println!("child {} exited with error code: {}", wr.pid, wr.exit_code);
                }
            },
        },
        Err(err) => panic!("fork failed: {:?}", err),
    }

    0
}
