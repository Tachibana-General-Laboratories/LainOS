#include "macro.S"

#define EL0 0b00
#define EL1 0b01
#define EL2 0b10
#define EL3 0b11

.section ".text.boot"

.global _start
_start:
    // enable CNTP for EL1 (core0)
    mrs     x0, cnthctl_el2
    orr     x0, x0, #3
    msr     cnthctl_el2, x0
    msr     cntvoff_el2, xzr

    // cpu id > 0, stop
    cpuid   x7
    cbz     x7, setup

halt:
    wfe
    b       halt

setup:

multicore_start:
    // set up EL1
    mrs     x0, CurrentEL
    and     x0, x0, #12 // clear reserved bits

    // running at EL3?
    cmp     x0, #12
    bne     start_el2

    // should never be executed, just for completeness
    mov     x2, #0x5b1
    msr     scr_el3, x2
    mov     x2, #0x3c9
    msr     spsr_el3, x2
    adr     x2, start_el2
    msr     elr_el3, x2
    eret

    // running at EL2?
start_el2:
    cmp     x0, #4
    beq     start_el1

    // init MPID/MPIDR
    mrs     x0, midr_el1
    mrs     x1, mpidr_el1
    msr     vpidr_el2, x0
    msr     vmpidr_el2, x1

    // enable AArch64 in EL1
    mov     x0, #(1 << 31)      // AArch64
    orr     x0, x0, #(1 << 1)   // SWIO hardwired on Pi3
    msr     hcr_el2, x0
    mrs     x0, hcr_el2

    // disable copro traps to EL2
    mov     x0, #0x33ff
    msr     cptr_el2, x0
    msr     hstr_el2, xzr

    // setup SCTLR access
    mov     x2, #0x0800
    movk    x2, #0x30d0, lsl #16
    orr     x0, x0, #(0x1 << 2)     // The C bit on (data cache).
    orr     x0, x0, #(0x1 << 12)    // The I bit on (instruction cache)
    msr     sctlr_el1, x2

/*
    cpuid   x7
    cbnz    x7, 1f
    bl      init_mmu
1:
*/

    // enable FP/SIMD at EL1
    mov     x0, #3 << 20
    msr     cpacr_el1, x0

    cpuid   x7

    // stack for cpu0
    ldr     x0, =_stack_core0_el0
    ldr     x1, =_stack_core0_el1
    ldr     x2, =_stack_core0_el1
    cmp     x7, #0
    beq     set_stacks

    // stack for cpu1
    ldr     x0, =_stack_core1_el0
    ldr     x1, =_stack_core1_el1
    ldr     x2, =_stack_core1_el1
    cmp     x7, #1
    beq     set_stacks

    // stack for cpu2
    ldr     x0, =_stack_core2_el0
    ldr     x1, =_stack_core2_el1
    ldr     x2, =_stack_core2_el1
    cmp     x7, #2
    beq     set_stacks

    // stack for cpu3
    ldr     x0, =_stack_core3_el0
    ldr     x1, =_stack_core3_el1
    ldr     x2, =_stack_core3_el1
    cmp     x7, #3
    beq     set_stacks

    // support only 4 cores
    b       halt

set_stacks:
    msr     sp_el0, x0
    msr     sp_el1, x1
    mov     sp    , x2

    // change execution level to EL1
    mov     x4, #0x3c5
    msr     spsr_el2, x4
    adr     x4, start_el1
    msr     elr_el2, x4
    eret

start_el1:
    // only for core0
    cpuid   x7
    cbnz    x7, rust_main

core0:
    // set up exception handlers
    ldr     x2, =_vector_table
    msr     vbar_el1, x2

    // clear bss
    ldr     x1, =__bss_start
    ldr     w2, =__bss_size
bss_clear_loop:
    cbz     w2, rust_main
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, bss_clear_loop

    // jump to C code, should not return
rust_main:
    cpuid   x0
    bl      kernel_main
    // for failsafe, halt this core
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

.macro HANDLER source, kind
.align 7
    stp     x30, x0, [SP, #-16]!
    mov     x0, \source
    movk    x0, \kind, LSL #16
    bl      context_save
    bl      context_restore
    ldp     x30, x0, [SP], #16
    eret
.endm

context_save:
    sub     SP, SP, #272

    stp     xzr, x29, [SP, #32 ]
    stp     x28, x27, [SP, #48 ]
    stp     x26, x25, [SP, #64 ]
    stp     x24, x23, [SP, #80 ]
    stp     x22, x21, [SP, #96 ]
    stp     x20, x19, [SP, #112]
    stp     x18, x17, [SP, #128]
    stp     x16, x15, [SP, #144]
    stp     x14, x13, [SP, #160]
    stp     x12, x11, [SP, #176]
    stp     x10, x9 , [SP, #192]
    stp     x8 , x7 , [SP, #208]
    stp     x6 , x5 , [SP, #224]
    stp     x4 , x3 , [SP, #240]
    stp     x2 , x1 , [SP, #256]

    // TODO: q31..q0

    mrs     x1, TPIDR_EL0
    mrs     x2, SP_EL0
    mrs     x3, SPSR_EL1
    mrs     x4, ELR_EL1

    stp     x1, x2, [SP, #0]
    stp     x3, x4, [SP, #16]

    mrs     x1, ESR_EL1
    mov     x2, SP

    enable_interrupts
    stp     x30, x0, [SP, #-16]!
    bl      handle_exception
    ldp     x30, x0, [SP], #16
    disable_interrupts

    ret

.global context_restore
context_restore:
    ldp     x1, x2, [SP, #0]
    ldp     x3, x4, [SP, #16]

    msr     TPIDR_EL0, x1
    msr     SP_EL0, x2
    msr     SPSR_EL1, x3
    msr     ELR_EL1, x4

    stp     xzr, x29, [SP, #32 ]
    stp     x28, x27, [SP, #48 ]
    stp     x26, x25, [SP, #64 ]
    stp     x24, x23, [SP, #80 ]
    stp     x22, x21, [SP, #96 ]
    stp     x20, x19, [SP, #112]
    stp     x18, x17, [SP, #128]
    stp     x16, x15, [SP, #144]
    stp     x14, x13, [SP, #160]
    stp     x12, x11, [SP, #176]
    stp     x10, x9 , [SP, #192]
    stp     x8 , x7 , [SP, #208]
    stp     x6 , x5 , [SP, #224]
    stp     x4 , x3 , [SP, #240]
    stp     x2 , x1 , [SP, #256]

    // TODO: q31..q0

    add     SP, SP, #272

    ret


.global xsvc
xsvc:
    #mov x8, #666
    svc #666
    ret


// macro to align handlers every 0x80 bytes
//.balign 0x80

.globl      _vector_table
.balign 0x800
_vector_table:
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
