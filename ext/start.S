#include "macro.S"

.section ".text.boot"

.global _start
_start:
    // enable CNTP for EL1 (core0)
    mrs     x0, cnthctl_el2
    orr     x0, x0, #3
    msr     cnthctl_el2, x0
    msr     cntvoff_el2, xzr

    cpuid   x7
    cbz     x7, 2f
    // cpu id > 0, stop
1:  wfe
    b       1b
2:  // cpu id == 0

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

    cpuid   x7
    cbnz    x7, 1f
    bl      init_mmu

1:

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
halt:
    wfe
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

.macro      start_handler el=el1
    str     x30,      [sp, #-16]!
    stp     x28, x29, [sp, #-16]!
    stp     x26, x27, [sp, #-16]!
    stp     x24, x25, [sp, #-16]!
    stp     x22, x23, [sp, #-16]!
    stp     x20, x21, [sp, #-16]!
    stp     x18, x19, [sp, #-16]!
    stp     x16, x17, [sp, #-16]!
    stp     x14, x15, [sp, #-16]!
    stp     x12, x13, [sp, #-16]!
    stp     x10, x11, [sp, #-16]!
    stp     x8 , x9 , [sp, #-16]!
    stp     x6 , x7 , [sp, #-16]!
    stp     x4 , x5 , [sp, #-16]!
    stp     x2 , x3 , [sp, #-16]!
    stp     x0 , x1 , [sp, #-16]!
    mrs     x1, spsr_\el
    mrs     x2, elr_\el
    stp     x1 , x2 , [sp, #-16]!
.endm

.macro      end_handler el=el1
    ldp     x1 , x2 , [sp], #16
    msr     elr_\el, x2
    msr     spsr_\el, x1
    ldp     x0 , x1 , [sp], #16
    ldp     x2 , x3 , [sp], #16
    ldp     x4 , x5 , [sp], #16
    ldp     x6 , x7 , [sp], #16
    ldp     x8 , x9 , [sp], #16
    ldp     x10, x11, [sp], #16
    ldp     x12, x13, [sp], #16
    ldp     x14, x15, [sp], #16
    ldp     x16, x17, [sp], #16
    ldp     x18, x19, [sp], #16
    ldp     x20, x21, [sp], #16
    ldp     x22, x23, [sp], #16
    ldp     x24, x25, [sp], #16
    ldp     x26, x27, [sp], #16
    ldp     x28, x29, [sp], #16
    ldr     x30,      [sp], #16
.endm

.macro      except num
    start_handler
    enable_interrupts

    mov     x0, \num
    mrs     x1, esr_el1
    mrs     x2, elr_el1
    mrs     x3, spsr_el1
    mrs     x4, far_el1
    mov     x5, sp
    bl      exception_handler

    disable_interrupts
    end_handler
    eret
.endm


.global xsvc
xsvc:
    #mov x8, #666
    svc #0
    ret

_xh0: except #0x0
_xh1: except #0x1
_xh2: except #0x2
_xh3: except #0x3

_xh4: except #0x4
_xh5: except #0x5
_xh6: except #0x6
_xh7: except #0x7

_xh8: except #0x8
_xh9: except #0x9
_xhA: except #0xA
_xhB: except #0xB

_xhC: except #0xC
_xhD: except #0xD
_xhE: except #0xE
_xhF: except #0xF


_ert:
    eret
.balign 4
.ltorg // Tell assembler ltorg data for code above can go here



// macro to align handlers every 0x80 bytes
.macro      vector handler
.balign 0x80
    b       \handler
.endm

.globl      _vector_table
.balign 0x800
_vector_table:
    // from current EL with sp_el0
    vector  _xh0    // synchronous
    vector  _xh1    // IRQ
    vector  _xh2    // FIQ
    vector  _xh3    // SError

    // from current EL with sp_elx, x != 0
    vector  _xh4    // Synchronous
    vector  _xh5    // IRQ
    vector  _xh6    // FIQ
    vector  _xh7    // SError

    // from lower EL, target EL minus 1 is AArch64
    vector  _xh8    // Synchronous
    vector  _xh9    // IRQ
    vector  _xhA    // FIQ
    vector  _xhB    // SError

    // from lower EL, target EL minus 1 is AArch32
    vector  _xhC    // Synchronous
    vector  _xhD    // IRQ
    vector  _xhE    // FIQ
    vector  _xhF    // SError
