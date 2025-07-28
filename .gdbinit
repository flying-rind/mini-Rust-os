target remote :1234
symbol-file Ncore/target/x86_64/debug/Ncore
b Ncore::kernel_main_monolithic
#b Ncore::kernel_main_hybrid
#b monolithic_syscalls::proc::<impl monolithic_syscalls::Syscall>::sys_exec
b hybrid_syscalls::task::<impl hybrid_syscalls::Syscall>::sys_exec
#b monolithic_syscalls::proc::<impl monolithic_syscalls::Syscall>::sys_fork
#b monolithic_syscalls::proc::<impl monolithic_syscalls::Syscall>::sys_wait4
#b monolithic_syscalls::fs::<impl monolithic_syscalls::Syscall>::sys_write
#b loader/src/monolithic.rs:60

define record_ins
while 1
x/i $rip
si
if $rip == 0x2020f6
break
end
end
end

define to_file
set logging overwrite on
set logging redirect on
set logging file gdb_trace.log
set logging on
end

layout src
c
