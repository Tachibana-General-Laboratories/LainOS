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

.section ".text.boot"

.global _start
_start:
    // cpu id > 0, stop
    cpuid   x7
    cbz     x7, setup

halt:
    wfe
    b       halt

setup:
    // read the current exception level into x0 (ref: C5.2.1)
    mrs     x6, CurrentEL
    and     x6, x6, #0b1100
    lsr     x6, x6, #2

    ldr     x8, =_stack_core0_el0
    ldr     x9, =_stack_core0_el1
    ldr     x10, =_stack_core0_el2

switch_to_el2:
    // switch to EL2 if we're in EL3. otherwise switch to EL1
    cmp     x6, EL3
    bne     switch_to_el1

    // set-up SCR_EL3 (bits 0, 4, 5, 7, 8, 10) (A53: 4.3.42)
    mov     x2, #0x5b1
    msr     SCR_EL3, x2

    // set-up SPSR and PL switch! (bits 0, 3, 6, 7, 8, 9) (ref: C5.2.20)
    mov     x2, #0x3c9
    msr     SPSR_EL3, x2
    adr     x2, switch_to_el1
    msr     ELR_EL3, x2
    eret

    // running at EL2?
switch_to_el1:
    // switch to EL1 if we're not already in EL1. otherwise continue with start
    cmp     x6, EL1
    beq     set_stack

    // init MPID/MPIDR
    mrs     x0, midr_el1
    mrs     x1, mpidr_el1
    msr     vpidr_el2, x0
    msr     vmpidr_el2, x1

    // disable copro traps to EL2
    mov     x0, #0x33ff
    msr     cptr_el2, x0
    msr     hstr_el2, xzr

    // set the stack-pointer for EL1
    msr     SP_EL1, x9

    // set the stack-pointer for EL2
    mov     SP    , x10

    // enable CNTP for EL1/EL0 (ref: D7.5.2, D7.5.13)
    // NOTE: This doesn't actually enable the counter stream.
    mrs     x0, CNTHCTL_EL2
    orr     x0, x0, #0b11
    msr     CNTHCTL_EL2, x0
    msr     CNTVOFF_EL2, xzr

    // enable AArch64 in EL1 (A53: 4.3.36)
    mov     x0, #(1 << 31)      // Enable AArch64 for EL1
    orr     x0, x0, #(1 << 1)   // RES1 on A-53
    msr     HCR_EL2, x0
    mrs     x0, HCR_EL2

    // enable floating point and SVE (SIMD) (A53: 4.3.38, 4.3.34)
    msr     CPTR_EL2, xzr     // don't trap accessing SVE registers
    mrs     x0, CPACR_EL1
    orr     x0, x0, #(0b11 << 20)
    msr     CPACR_EL1, x0

    // Set SCTLR to known state (RES1: 11, 20, 22, 23, 28, 29) (A53: 4.3.30)
    mov     x2, #0x0800
    movk    x2, #0x30d0, lsl #16
    msr     SCTLR_EL1, x2

    // set up exception handlers
    ldr     x2, =_vector_table_el1
    msr     VBAR_EL1, x2

    // change execution level to EL1
    mov     x4, #0x3c5
    msr     SPSR_EL2, x4
    adr     x4, set_stack
    msr     ELR_EL2, x4
    eret

set_stack:
    msr     SP_EL0, x8
    mov     SP    , x9

zero_bss:
    // load the start address and number of bytes in BSS section
    ldr     x1, =__bss_start
    ldr     x2, =__bss_size
zero_bss_loop:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, go_main
    str     xzr, [x1], #8
    sub     x2, x2, #1
    cbnz    x2, zero_bss_loop

go_main:
    bl      kernel_main
    b       halt


// how about it:
//  32 bit float registers: S0 ... S31
//  64 bit double registers: D0 ... D31
//  128 bit SIMD registers: V0 ... V31

//  V0 - V7 arguments and return value
//  D8 - D15 callee saved registers
//  V16 - V31 temporary registers
// Bits 64:128 not saved on V8-V15

//  X0 - X7 arguments and return value
//  X8 indirect result (struct) location
//  X9 - X15 temporary registers
//  X16 - X17 intra-call-use registers (PLT, linker)
//  X18 platform specific use (TLS)
//  X19 - X28 callee-saved registers
//  X29 frame pointer
//  X30 link register
//  SP stack pointer (XZR)

.macro HANDLER source, kind, call=handle_exception
.align 7
    stp     x30, x0, [SP, #-16]!
    mov     x0, \source
    movk    x0, \kind, LSL #16
    bl      context_save
    //enable_interrupts
    mrs     x1, ESR_EL1
    mov     x2, SP
    bl      \call
    //disable_interrupts
    bl      context_restore
    ldp     x30, x0, [SP], #16
    eret
.endm

context_save:
    sub     SP, SP, #272

    stp     x1 , x2 , [SP, #32 ]
    stp     x3 , x4 , [SP, #48 ]
    stp     x5 , x6 , [SP, #64 ]
    stp     x7 , x8 , [SP, #80 ]
    stp     x9 , x10, [SP, #96 ]
    stp     x11, x12, [SP, #112]
    stp     x13, x14, [SP, #128]
    stp     x15, x16, [SP, #144]
    stp     x17, x18, [SP, #160]
    stp     x19, x20, [SP, #176]
    stp     x21, x22, [SP, #192]
    stp     x23, x24, [SP, #208]
    stp     x25, x26, [SP, #224]
    stp     x27, x28, [SP, #240]
    stp     x29, xzr, [SP, #256]

    // TODO: q31..q0

    mrs     x1, TPIDR_EL0
    mrs     x2, SP_EL0
    mrs     x3, SPSR_EL1
    mrs     x4, ELR_EL1

    stp     x4, x3, [SP, #0]
    stp     x2, x1, [SP, #16]

    ret

.global context_restore
context_restore:
    ldp     x4, x3, [SP, #0]
    ldp     x2, x1, [SP, #16]

    msr     TPIDR_EL0, x1
    msr     SP_EL0, x2
    msr     SPSR_EL1, x3
    msr     ELR_EL1, x4

    // TODO: q31..q0

    ldp     x1 , x2 , [SP, #32 ]
    ldp     x3 , x4 , [SP, #48 ]
    ldp     x5 , x6 , [SP, #64 ]
    ldp     x7 , x8 , [SP, #80 ]
    ldp     x9 , x10, [SP, #96 ]
    ldp     x11, x12, [SP, #112]
    ldp     x13, x14, [SP, #128]
    ldp     x15, x16, [SP, #144]
    ldp     x17, x18, [SP, #160]
    ldp     x19, x20, [SP, #176]
    ldp     x21, x22, [SP, #192]
    ldp     x23, x24, [SP, #208]
    ldp     x25, x26, [SP, #224]
    ldp     x27, x28, [SP, #240]
    ldp     x29, xzr, [SP, #256]

    add     SP, SP, #272

    ret

// macro to align handlers every 0x80 bytes
//.balign 0x80

.globl      _vector_table_el1
.balign 0x800
_vector_table_el1:
    // Target and source at same exception level with source SP = SP_EL0
    HANDLER #0x0, #0x0  // Synchronous exception
    HANDLER #0x0, #0x1  // IRQ
    HANDLER #0x0, #0x2  // FIQ
    HANDLER #0x0, #0x3  // SError

    // Target and source at same exception level with source SP = SP_ELx
    HANDLER #0x1, #0x0  // Synchronous exception
    HANDLER #0x1, #0x1  // IRQ
    HANDLER #0x1, #0x2  // FIQ
    HANDLER #0x1, #0x3  // SError

    // Source is at lower exception level running on AArch64
    HANDLER #0x2, #0x0  // Synchronous exception
    HANDLER #0x2, #0x1  // IRQ
    HANDLER #0x2, #0x2  // FIQ
    HANDLER #0x2, #0x3  // SError

    // Source is at lower exception level running on AArch32
    HANDLER #0x3, #0x0  // Synchronous exception
    HANDLER #0x3, #0x1  // IRQ
    HANDLER #0x3, #0x2  // FIQ
    HANDLER #0x3, #0x3  // SError
