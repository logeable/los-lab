#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use user::{console::Stdin, entry, exec, fork, print, println, waitpid};

entry!(main);

const LF: u8 = 0x0a;
const BS: u8 = 0x08;

fn main() -> i32 {
    println!("welcome to lshell");
    prompt();

    let mut line = String::new();
    loop {
        let c = Stdin::read_u8().unwrap();

        match c {
            LF => {
                if !line.is_empty() {
                    let program = line.trim();
                    match fork().expect("fork must succeed") {
                        user::ForkProc::Child => {
                            if let Err(e) = exec(program) {
                                println!("exec {} failed: {}", line, e);
                            }
                        }
                        user::ForkProc::Parent(pid) => {
                            println!("subprocess {}({}) started", program, pid);
                            let wr = waitpid(pid).expect("waitpid must succeed");
                            assert_eq!(pid, wr.pid);

                            println!(
                                "subprocess {}({}) exited with {}",
                                program, pid, wr.exit_code
                            )
                        }
                    }
                }
                prompt();
            }
            BS => {
                if !line.is_empty() {
                    line.pop();
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                }
            }
            _ => {
                line.push(c as char);
                print!("{}", c as char);
            }
        }
    }
}

fn prompt() {
    print!(">> ");
}
