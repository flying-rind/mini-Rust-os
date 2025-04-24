#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::close;
use user_lib::println;
use user_lib::proc_wait;
use user_lib::read;
use user_syscall::fork;
use user_syscall::make_pipe;
use user_syscall::write;

static STR: &str = "Hello, world!";

#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> usize {
    let (read_fd, write_fd) = make_pipe();
    // println!("read_fd: {}, write_fd:{}", read_fd, write_fd);
    let pid = fork();
    if pid == 0 {
        println!("pid: {}", pid);
        // 子进程，关闭写端，再读读端
        close(write_fd);
        let mut buffer = [0u8; 32];
        let read_size = read(read_fd, &mut buffer).unwrap();
        // 关闭读端
        close(read_fd);
        // let a = 1;
        println!("read_size: {}", read_size);
        assert_eq!(core::str::from_utf8(&buffer[..read_size]).unwrap(), STR);
        0
    } else {
        println!("child pid: {}", pid);
        // 父进程，关闭读端，再写写端
        close(read_fd);
        assert_eq!(write(write_fd, STR.as_bytes()).unwrap(), STR.len());
        // 关闭写端
        close(write_fd);
        proc_wait(pid);
        println!("pipetest passed!");
        0
    }
}
