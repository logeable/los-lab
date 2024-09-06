.altmacro

.macro SAVE_S i
    sd s\i, 16+\i*8(t0)
.endm

.macro LOAD_S i
    ld s\i, 16+\i*8(t0)
.endm
    
    .text
    .align 2
_switch_task:
    mv t0, a0
    sd ra, 0*8(t0)
    sd sp, 1*8(t0)
    .set i, 0
    .rept 12
        SAVE_S %i
        .set i, i+1
    .endr
    

    mv t0, a1
    ld ra, 0*8(t0)
    ld sp, 1*8(t0)
    .set i, 0
    .rept 12
        LOAD_S %i
        .set i, i+1
    .endr
    
    ret
