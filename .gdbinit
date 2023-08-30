target remote :1234
symbol-file kernel/target/x86_64-unknown-none/debug/kernel -o 0x0
b kernel_main
c
layout asm
b kernel::syscall::process::sys_exec
c
// b kernel::process::task::Task::exit
// c
// b kernel::syscall::syscall
// c
