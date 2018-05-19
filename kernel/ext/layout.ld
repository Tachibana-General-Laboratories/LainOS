SECTIONS {
    . = 0x80000;
    .text : {
        KEEP(*(.text.boot))
        . = . + 0x10000;
        *(.text .text.* .gnu.linkonce.t*)
    }

    .rodata : {
        *(.rodata .rodata.* .gnu.linkonce.r*)
    }
    . = ALIGN(4096);

    PROVIDE(_data = .);
    .data : {
        *(.data .data.* .gnu.linkonce.d*)
    }

    . = ALIGN(4096);

    .bss (NOLOAD) : {
        . = ALIGN(32);
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)

        . = ALIGN(16);

        _stack_core0_start = .;
        . = . + 0x00100000;
        _stack_core0_el1 = .;

        . = ALIGN(32);
        __bss_end = .;
    }

    . = ALIGN(4096);
    _end = .;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
__bss_size = (__bss_end - __bss_start)>>3;
