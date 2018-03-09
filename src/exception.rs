use console::{kprint, kprintln};
use volatile::prelude::*;
use volatile::{Volatile, Reserved};

#[repr(C)]
#[derive(Debug)]
struct State {
    spsr: Volatile<u64>,
    elr: Volatile<u64>,
    reg: [Volatile<u64>; 32],
}

// common exception handler
#[no_mangle]
#[inline(never)]
pub extern "C" fn exception_handler(kind: u64, esr: u64, elr: u64, spsr: u64, far: u64, sp: u64) {
    let esr = esr as u32;
    let spsr = spsr as u32;

    let mut state = unsafe { &mut *((sp) as *mut State) };

    warn!("IT'S A TRAP!");

    // print out interruption type
    match kind & 3 {
        0 => kprint!("!!! Synchronous {}", kind),
        1 => kprint!("!!! IRQ {}", kind),
        2 => kprint!("!!! FIQ {}", kind),
        3 => kprint!("!!! SError {}", kind),
        _ => unreachable!(),
    }

    kprint!(": ");

    let ec = esr >> 26;

    // decode exception type (some, not all. See ARM DDI0487B_b chapter D10.2.28)
    match ec {
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
    if ec == 0b100100 || ec == 0b100101 {
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
    debug!("reg el1:  ESR {:08X}  ELR {:016X} SPSR {:016X}  FAR {:016X}", esr, elr, spsr, far);

    if kind != 8 {
        // no return from exception for now
        loop {}
    } else {
        let svc_imm = esr as u16;
        state.reg[0].write(1488);
    }

    return;
}
