.altmacro

.macro SAVE_X i
    sd x\i, \i*8(sp)
.endm

.macro LOAD_X i
    ld x\i, \i*8(sp)
.endm

    .section .text
    .align 2
    .globl _s_trap_enter
_s_trap_enter:
    csrrw sp, sscratch, sp
    add sp, sp, -34*8

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

    mv a0, sp
    call process_trap

    .globl _s_trap_return
_s_trap_return:
    ld t0, 2*8(sp)
    csrw sscratch, t0
    ld t0, 32*8(sp)
    csrw sstatus, t0
    ld t0, 33*8(sp)
    csrw sepc, t0

    sd x1, 1*8(sp)    
    .set i, 3 
    .rept 29
        LOAD_X %i
        .set i, i+1
    .endr

    add sp, sp, 34*8
    csrrw sp, sscratch, sp
    sret