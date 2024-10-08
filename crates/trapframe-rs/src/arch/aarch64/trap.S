.section .text

.global __alltraps
__alltraps:
    # x30 and x0 are saved in __vectors
    # x0 is trap num now
    # skip __reversed
    str     x29, [sp, #-16]!
    stp     x27, x28, [sp, #-16]!
    stp     x25, x26, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!
    stp     x17, x18, [sp, #-16]!
    stp     x15, x16, [sp, #-16]!
    stp     x13, x14, [sp, #-16]!
    stp     x11, x12, [sp, #-16]!
    stp     x9, x10, [sp, #-16]!
    stp     x7, x8, [sp, #-16]!
    stp     x5, x6, [sp, #-16]!
    stp     x3, x4, [sp, #-16]!
    stp     x1, x2, [sp, #-16]!
    str     q3, [sp, #-16]!
    str     q2, [sp, #-16]!
    str     q1, [sp, #-16]!
    str     q0, [sp, #-16]!

    # skip tpidr and sp
    add     sp, sp, #-16

    # read spsr and elr
    mrs     x2, spsr_el1
    mrs     x1, elr_el1
    stp     x1, x2, [sp, #-16]!

    # save trap num
    str     x0, [sp, #-16]!

    # check source is 2
    mov     x1, #0x3
    and     x1, x1, x0
    cmp     x1, #2
    beq     trap_from_user

trap_from_kernel:
    # read tpidr and sp
    mrs     x2, tpidr_el1
    add     x1, sp, #46*8
    stp     x1, x2, [sp, #32]
    # go to rust
    mov     x0, sp
    bl      trap_handler

    # skip setting sp if returning 0
    cmp     x0, #0
    beq     skip_set_sp
    mov     sp, x0

skip_set_sp:
    # load tpidr
    ldr     x1, [sp, #5*8]
    msr     tpidr_el1, x1
    # go to trap_return
    b       trap_return

trap_from_user:
    # read and store tpidr and user sp
    mrs     x2, tpidr_el0
    mrs     x1, sp_el0
    stp     x1, x2, [sp, #4*8]
    # read and set kernel sp
    ldr     x2, [sp, #8]
    mov     sp, x2

    # load callee-saved registers
    ldp     x19, x20, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x23, x24, [sp], #16
    ldp     x25, x26, [sp], #16
    ldp     x27, x28, [sp], #16
    ldp     x29, x30, [sp], #16

    ret

.macro HANDLER source kind
    .align 7
    # sp is set to SP_EL1 upon trap
    stp     lr, x0, [sp, #-16]!
    mov     x0, #\source
    movk    x0, #\kind, lsl #16
    b       __alltraps
.endm

.global __vectors
.align 11
__vectors:
    HANDLER 0 0
    HANDLER 0 1
    HANDLER 0 2
    HANDLER 0 3
    HANDLER 1 0
    HANDLER 1 1
    HANDLER 1 2
    HANDLER 1 3
    HANDLER 2 0
    HANDLER 2 1
    HANDLER 2 2
    HANDLER 2 3
    HANDLER 3 0
    HANDLER 3 1
    HANDLER 3 2
    HANDLER 3 3

.global run_user
run_user:
    # x0 points to TrapFrame
    # save callee-saved registers x19-x29
    stp     x29, x30, [sp, #-16]!
    stp     x27, x28, [sp, #-16]!
    stp     x25, x26, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!

    # save kernel sp to TrapFrame
    mov     x1, sp
    mov     sp, x0
    str     x1, [sp, #8]

    # load sp and tpidr
    ldp     x1, x2, [sp, #4*8]
    msr     sp_el0, x1
    msr     tpidr_el0, x2

trap_return:
    # sp points to TrapFrame
    # skip trap num, don't restore
    add     sp, sp, #16

    # elr and spsr
    ldp     x1, x2, [sp], #16
    msr     elr_el1, x1
    msr     spsr_el1, x2

    # skip sp and tpidr
    add     sp, sp, #16

    # general purpose registers
    ldr     q0, [sp], #16
    ldr     q1, [sp], #16
    ldr     q2, [sp], #16
    ldr     q3, [sp], #16
    ldp     x1, x2, [sp], #16
    ldp     x3, x4, [sp], #16
    ldp     x5, x6, [sp], #16
    ldp     x7, x8, [sp], #16
    ldp     x9, x10, [sp], #16
    ldp     x11, x12, [sp], #16
    ldp     x13, x14, [sp], #16
    ldp     x15, x16, [sp], #16
    ldp     x17, x18, [sp], #16
    ldp     x19, x20, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x23, x24, [sp], #16
    ldp     x25, x26, [sp], #16
    ldp     x27, x28, [sp], #16
    ldr     x29, [sp], #16
    ldp     lr, x0, [sp], #16

    # return
    eret