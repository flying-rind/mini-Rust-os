target remote :1234
symbol-file kernel/target/x86_64/debug/kernel
b kernel_main
c
# b kernel::syscall::proc::sys_fork
b sys_exec
b sys_write
c
layout src

