target remote :1234
symbol-file Ncore/target/x86_64/debug/Ncore
b Ncore::kernel_main_monolithic
b executor::run_until_idle
# b trapframe::arch::UserContext::run
layout src
c
