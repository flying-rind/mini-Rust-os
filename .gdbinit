target remote :1234
symbol-file kernel/target/x86_64/debug/kernel
#b kernel::kernel_main
b kernel/src/main.rs:60
layout src
c
