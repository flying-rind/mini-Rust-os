target remote :1234
symbol-file kernel/target/x86_64-unknown-none/debug/kernel -o 0x8000000000
b kernel::init
layout src
c

