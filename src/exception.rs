use console::{kprint, kprintln};

// common exception handler
#[no_mangle]
#[inline(never)]
pub extern "C" fn exception_handler(kind: u64, esr: u64, elr: u64, spsr: u64, far: u64) -> ! {
    // print out interruption type
    match kind {
        0 => kprint!("Synchronous"),
        1 => kprint!("IRQ"),
        2 => kprint!("FIQ"),
        3 => kprint!("SError"),
        v => kprint!("unknown [{}]", v),
    }

    kprint!(": ");

    // decode exception type (some, not all. See ARM DDI0487B_b chapter D10.2.28)
    match esr >> 26 {
        0b000000 => kprint!("Unknown reason"),
        0b000001 => kprint!("Trapped WFI/WFE"),
        0b000111 => kprint!("Access to SVE/adv SIMD/float"),
        0b001110 => kprint!("Illegal execution"),
        0b010101 => kprint!("System call"),
        0b100000 => kprint!("Instruction abort, lower EL"),
        0b100001 => kprint!("Instruction abort, same EL"),
        0b100010 => kprint!("Instruction alignment fault"),
        0b100100 => kprint!("Data abort, lower EL"),
        0b100101 => kprint!("Data abort, same EL"),
        0b100110 => kprint!("Stack alignment fault"),
        0b101100 => kprint!("Floating point"),
        v => kprint!("Unknown 0b{:06b}", v),
    }

    // decode data abort cause
    if esr>>26 == 0b100100 || esr>>26 == 0b100101 {
        kprint!(", ");
        match (esr>>2) & 0x3 {
            0 => kprint!("Address size fault"),
            1 => kprint!("Translation fault"),
            2 => kprint!("Access flag fault"),
            3 => kprint!("Permission fault"),
            _ => unreachable!(),
        }
        kprint!(" at level {}", esr & 0x3);
    }
    kprintln!("");

    // dump registers
    kprintln!("  ESR_EL1 {:016X}", esr);
    kprintln!("  ELR_EL1 {:016X}", elr);
    kprintln!(" SPSR_EL1 {:016X}", spsr);
    kprintln!("  FAR_EL1 {:016X}", far);

    // no return from exception for now
    loop {}
}
