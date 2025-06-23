target remote :1234
symbol-file Ncore/target/x86_64/debug/Ncore
b Ncore::kernel_main_monolithic
b monolithic_syscalls::proc::<impl monolithic_syscalls::Syscall>::sys_exec
#b monolithic_syscalls::proc::<impl monolithic_syscalls::Syscall>::sys_fork
#b monolithic_syscalls::proc::<impl monolithic_syscalls::Syscall>::sys_wait4
#b monolithic_syscalls::fs::<impl monolithic_syscalls::Syscall>::sys_write
#b loader/src/monolithic.rs:48
layout src
c
