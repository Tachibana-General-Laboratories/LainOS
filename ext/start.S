.section ".text.boot"

.global _start

_start:
    // read cpu id, stop slave cores
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, 2f
    // cpu id > 0, stop
1:  wfe
    b       1b
2:  // cpu id == 0

    // set stack before our code
    ldr     x1, =_start

    // set up EL1
    mrs     x0, CurrentEL
    and     x0, x0, #12 // clear reserved bits


    // running at EL3?
    cmp     x0, #12
    bne     5f
    // should never be executed, just for completeness
    mov     x2, #0x5b1
    msr     scr_el3, x2
    mov     x2, #0x3c9
    msr     spsr_el3, x2
    adr     x2, 5f
    msr     elr_el3, x2
    eret

    // running at EL2?
5:  cmp     x0, #4
    beq     5f
    msr     sp_el1, x1
    // enable CNTP for EL1
    mrs     x0, cnthctl_el2
    orr     x0, x0, #3
    msr     cnthctl_el2, x0
    msr     cntvoff_el2, xzr
    // enable AArch64 in EL1
    mov     x0, #(1 << 31)      // AArch64
    orr     x0, x0, #(1 << 1)   // SWIO hardwired on Pi3
    msr     hcr_el2, x0
    mrs     x0, hcr_el2
    // Setup SCTLR access
    mov     x2, #0x0800
    movk    x2, #0x30d0, lsl #16
    msr     sctlr_el1, x2
    // change execution level to EL1
    mov     x2, #0x3c4
    msr     spsr_el2, x2
    adr     x2, 5f
    msr     elr_el2, x2
    eret

5:  mov     sp, x1

    // set up exception handlers
    ldr     x2, =_vector_table
    msr     vbar_el1, x2

    // clear bss
    ldr     x1, =__bss_start
    ldr     w2, =__bss_size
3:  cbz     w2, 4f
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, 3b

    // enable FP/SIMD at EL1
    mov     x0, #3 << 20
    msr     cpacr_el1, x0

    // jump to C code, should not return
4:  bl      kernel_main
    // for failsafe, halt this core too
    b       1b



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
    stp     x29, x30, [sp, #-16]!
    stp     x27, x28, [sp, #-16]!
    stp     x25, x26, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!
    stp     x17, x18, [sp, #-16]!
    stp     x15, x16, [sp, #-16]!
    stp     x13, x14, [sp, #-16]!
    stp     x11, x12, [sp, #-16]!
    stp     x9 , x10, [sp, #-16]!
    stp     x7 , x8 , [sp, #-16]!
    stp     x5 , x6 , [sp, #-16]!
    stp     x3 , x4 , [sp, #-16]!
    stp     x1 , x2 , [sp, #-16]!
    str          x0 , [sp, #-16]!

    mrs     x1, spsr_\el
    mrs     x2, elr_\el

    stp     x1 , x2 , [sp, #-16]!
.endm


.macro      end_handler el=el1
    ldp      x1,  x2, [sp], #16

    msr     elr_\el, x2
    msr     spsr_\el, x1

    ldr          x0 , [sp], #16
    ldp     x1 , x2 , [sp], #16
    ldp     x3 , x4 , [sp], #16
    ldp     x5 , x6 , [sp], #16
    ldp     x7 , x8 , [sp], #16
    ldp     x9 , x10, [sp], #16
    ldp     x11, x12, [sp], #16
    ldp     x13, x14, [sp], #16
    ldp     x15, x16, [sp], #16
    ldp     x17, x18, [sp], #16
    ldp     x19, x20, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x23, x24, [sp], #16
    ldp     x25, x26, [sp], #16
    ldp     x27, x28, [sp], #16
    ldp     x29, x30, [sp], #16
.endm

.macro      except num
    start_handler
    mov     x0, \num
    mrs     x1, esr_el1
    mrs     x2, elr_el1
    mrs     x3, spsr_el1
    mrs     x4, far_el1
    mov     x5, sp
    bl      exception_handler
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

hang:
    b hang
.balign 4
.ltorg // Tell assembler ltorg data for code above can go here

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
    vector  _ert    // Synchronous
    vector  _ert    // IRQ
    vector  _ert    // FIQ
    vector  _ert    // SError
