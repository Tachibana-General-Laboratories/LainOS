
// common exception handler
#[no_mangle]
#[inline(never)]
pub extern "C" fn exception_handler(kind: u64, esr: u64, elr: u64, spsr: u64, far: u64) -> ! {
    // print out interruption type
    match kind {
        0 => print!("Synchronous"),
        1 => print!("IRQ"),
        2 => print!("FIQ"),
        3 => print!("SError"),
        _ => print!("unknown"),
    }

    print!(": ");

    // decode exception type (some, not all. See ARM DDI0487B_b chapter D10.2.28)
    match esr >> 26 {
        0b000000 => print!("Unknown"),
        0b000001 => print!("Trapped WFI/WFE"),
        0b001110 => print!("Illegal execution"),
        0b010101 => print!("System call"),
        0b100000 => print!("Instruction abort, lower EL"),
        0b100001 => print!("Instruction abort, same EL"),
        0b100010 => print!("Instruction alignment fault"),
        0b100100 => print!("Data abort, lower EL"),
        0b100101 => print!("Data abort, same EL"),
        0b100110 => print!("Stack alignment fault"),
        0b101100 => print!("Floating point"),
        _ => print!("Unknown"),
    }

    // decode data abort cause
    if esr>>26 == 0b100100 || esr>>26 == 0b100101 {
        print!(", ");
        match (esr>>2) & 0x3 {
            0 => print!("Address size fault"),
            1 => print!("Translation fault"),
            2 => print!("Access flag fault"),
            3 => print!("Permission fault"),
            _ => unreachable!(),
        }
        print!(" at level {}", esr & 0x3);
    }
    println!("");

    // dump registers
    println!("  ESR_EL1 {:016X}", esr);
    println!("  ELR_EL1 {:016X}", elr);
    println!(" SPSR_EL1 {:016X}", spsr);
    println!("  FAR_EL1 {:016X}", far);

    // no return from exception for now
    loop {}
}
