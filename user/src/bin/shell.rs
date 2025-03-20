#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate user_lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

use alloc::string::String;
use user_lib::console::getchar;
use user_lib::{exec, fork, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    let mut string: String = String::new();
    print!("ffos$ ");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !string.is_empty() {
                    string.push('\0');
                    let pid = fork();
                    if pid == 0 {
                        // child process
                        if exec(string.as_str()) == -1 {
                            println!("Error when executing!");
                            return -4;
                        }
                        unreachable!();
                    } else {
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("Shell: Process {} exited with code {}", pid, exit_code);
                    }
                    string.clear();
                }
                print!(">> ");
            }
            BS | DL => {
                if !string.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    string.pop();
                }
            }
            _ => {
                print!("{}", c as char);
                string.push(c as char);
            }
        }
    }
}
