.altmacro

.macro SAVE_X i
    sd x\i, \i*8(sp)
.endm

.macro LOAD_X i
    ld x\i, \i*8(sp)
.endm

    .section .text.trampoline
    .align 2
    .globl _s_trap_enter
_s_trap_enter:
    csrrw sp, sscratch, sp

    .set i, 0
    .rept 32
        SAVE_X %i
        .set i, i+1
    .endr

    csrr t0, sscratch
    sd t0, 2*8(sp)
    csrr t0, sstatus
    sd t0, 32*8(sp)
    csrr t0, sepc
    sd t0, 33*8(sp)

    # kernel_satp
    ld t0, 34*8(sp) 
    # kernel_sp
    ld t1, 35*8(sp)
    # trap_handler
    ld t2, 36*8(sp)

    csrw satp, t0
    sfence.vma

    mv sp, t1
    jr t2

    .globl _s_trap_return
_s_trap_return:
    # a0: pointer to trap context
    # a1: app satp
    csrw sscratch, a0
    csrw satp, a1
    sfence.vma

    mv sp, a0
    ld t0, 32*8(sp)
    csrw sstatus, t0
    ld t0, 33*8(sp)
    csrw sepc, t0

    ld x1, 1*8(sp)
    .set i, 3 
    .rept 29
        LOAD_X %i
        .set i, i+1
    .endr

    ld sp, 2*8(sp)
    sret