SECTIONS
{
    . = 0x80000;
    .text : {
        KEEP(*(.text.boot))
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

    .bss (NOLOAD) : {
        . = ALIGN(16);
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)

        /* Stack must always be aligned to 8 byte boundary AAPCS64 call standard */
        . = ALIGN(16);

        _stack_core0_start = .;
        . = . + 0x4000;
        _stack_core0_el0 = .;
        . = . + 0x4000;
        _stack_core0_el1 = .;
        . = . + 0x400;
        _stack_core0_el2 = .;

        _stack_core1_start = .;
        . = . + 0x4000;
        _stack_core1_el0 = .;
        . = . + 0x4000;
        _stack_core1_el1 = .;
        . = . + 0x400;
        _stack_core1_el2 = .;

        _stack_core2_start = .;
        . = . + 0x4000;
        _stack_core2_el0 = .;
        . = . + 0x4000;
        _stack_core2_el1 = .;
        . = . + 0x400;
        _stack_core2_el2 = .;

        _stack_core3_start = .;
        . = . + 0x4000;
        _stack_core3_el0 = .;
        . = . + 0x4000;
        _stack_core3_el1 = .;
        . = . + 0x400;
        _stack_core3_el2 = .;

        . = ALIGN(4096);
        _ttbr_start = .;
        /* 4096 * 6 */
        . = . + 24576;

        __bss_end = .;
    }

    . = ALIGN(4096);
    _end = .;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
__bss_size = (__bss_end - __bss_start)>>3;
