.macro      disable_interrupts
    msr     daifset, #0xf
.endm

.macro      enable_interrupts
    msr     daifclr, #0xf
.endm

.macro      cpuid reg
    mrs     \reg, mpidr_el1     // Read core id on ARM8
    and     \reg, \reg, #0x3    // Make cpu id bitmask
.endm
