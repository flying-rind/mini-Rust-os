target remote :1234
symbol-file kernel/target/x86_64-unknown-none/debug/kernel

b kernel_main
layout src
c

