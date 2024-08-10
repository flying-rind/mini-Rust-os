target remote :1234
symbol-file kernel/target/x86_64/debug/kernel
b kernel_main
c
b kernel/src/lib.rs:138
layout src

