OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        KEEP(*(.text.entry))
        . = ALIGN(4K);
        strampoline = .;
        KEEP(*(.text.trampoline))
        . = ALIGN(4K);
        etrampoline = .;
        *(.text .text.*)
    }
    . = ALIGN(4K);
    etext = .;

    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
    }    
    . = ALIGN(4K);
    erodata = .;

    sdata = .;
    .data : {
        *(.data .data.*)
    }
    . = ALIGN(4K);
    edata = .;

    .bss : {
        sbtstack = .;
        *(.bss.stack)
        ebtstack = .;
        sbss = .;
        *(.bss .bss.*)
    }
    . = ALIGN(4K);
    ebss = .;

    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
