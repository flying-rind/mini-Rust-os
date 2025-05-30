target remote :1234
symbol-file Ncore/target/x86_64/debug/Ncore
b loader::hybrid::kernel_main
b hybrid_objects::task::process::Process::exec
# b trapframe::arch::UserContext::run
layout src
c
