#define EL0 0b00
#define EL1 0b01
#define EL2 0b10
#define EL3 0b11

.macro      disable_interrupts
    msr     DAIFSET, #0xf
.endm

.macro      enable_interrupts
    msr     DAIFCLR, #0xf
.endm

.macro      cpuid reg
    mrs     \reg, MPIDR_EL1     // Read core id on ARM8
    and     \reg, \reg, #0x3    // Make cpu id bitmask
.endm

.macro HANDLER source, kind, call=handle_exception
.align 7
    stp     x30, x0, [SP, #-16]!
    mov     x0, \source
    movk    x0, \kind, LSL #16
    bl      context_save
    enable_interrupts
    bl      \call
    disable_interrupts
    bl      context_restore
    ldp     x30, x0, [SP], #16
    eret
.endm
