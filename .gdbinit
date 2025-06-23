target remote :1234
symbol-file Ncore/target/x86_64/debug/Ncore
b Ncore::kernel_main_monolithic
#b monolithic_syscalls::proc::<impl monolithic_syscalls::Syscall>::sys_exec
b monolithic_syscalls::proc::<impl monolithic_syscalls::Syscall>::sys_fork
layout src
c
