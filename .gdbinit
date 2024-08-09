target remote :1234
symbol-file kernel/target/x86_64/debug/kernel
b kernel_main
c
b kernel::syscall::sync::sys_mutex_unlock
layout src

