target remote :1234
symbol-file kernel/target/x86_64-unknown-none/debug/kernel -o 0x0
b kernel_main
c
b kernel::app::loader::get_app_data_by_name
c
// b kernel::syscall::process::sys_exec
// c
layout src