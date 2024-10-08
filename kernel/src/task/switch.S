.text
.global 
context_switch: # (cur: &mut Context, nxt: &Context)
  # Save cur's registers
  mov rax, [rsp] # return address
  mov [rdi + 56], rax # 56 = offsetof(Context, rip)
  mov [rdi + 0], rsp
  mov [rdi + 8], rbx
  mov [rdi + 16], rbp
  mov [rdi + 24], r12
  mov [rdi + 32], r13
  mov [rdi + 40], r14
  mov [rdi + 48], r15
  # Restore nxt's registers
  mov rsp, [rsi + 0]
  mov rbx, [rsi + 8]
  mov rbp, [rsi + 16]
  mov r12, [rsi + 24]
  mov r13, [rsi + 32]
  mov r14, [rsi + 40]
  mov r15, [rsi + 48]
  mov rax, [rsi + 56] # restore return address
  mov [rsp], rax # for stack balance, must use mov instead of push
  ret

# Restore next kthread without saving current's context(used when reboot kthread)
.text
.global
restore_next: # (nxt: &KernelContext)
  mov rsp, [rdi + 0]
  mov rbx, [rdi + 8]
  mov rbp, [rdi + 16]
  mov r12, [rdi + 24]
  mov r13, [rdi + 32]
  mov r14, [rdi + 40]
  mov r15, [rdi + 48]
  mov rax, [rdi + 56] # restore return address
  mov [rsp], rax # for stack balance, must use mov instead of push
  ret