target remote :1234
symbol-file kernel/target/x86_64/debug/kernel
b kernel_main
c
b kernel::task::process::Process::exec
layout src

