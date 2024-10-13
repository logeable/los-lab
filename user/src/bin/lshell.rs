#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use user::{console::Stdin, entry, exec, fork, getpid, print, println, waitpid};

entry!(main);

const LF: u8 = 0x0a;
const CR: u8 = 0x0d;
const BS: u8 = 0x08;
const DEL: u8 = 0x07f;

fn main() -> i32 {
    println!("welcome to lshell");
    prompt();

    let mut line = String::new();
    loop {
        let c = Stdin::read_u8().unwrap();

        match c {
            CR | LF => {
                println!("");
                if !line.is_empty() {
                    let program = line.trim().to_string();
                    line.clear();

                    match fork().expect("fork must succeed") {
                        user::ForkProc::Child => {
                            if let Err(e) = exec(&program) {
                                panic!("exec {:?} failed: {}", program, e);
                            }
                        }
                        user::ForkProc::Parent(pid) => {
                            let wr = waitpid(pid).expect("waitpid must succeed");
                            assert_eq!(pid, wr.pid);

                            if wr.exit_code != 0 {
                                println!(
                                    "subprocess {}({}) exited with {}",
                                    program, pid, wr.exit_code
                                )
                            }
                        }
                    }
                }
                prompt();
            }
            BS | DEL => {
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
    print!("[{}] >> ", getpid());
}
