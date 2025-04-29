target remote :1234
symbol-file kernel/target/x86_64/debug/kernel
b kernel::kernel_main
#b kernel::fs::inode::init
#b kernel/src/fs/inode.rs:129
layout src
c
