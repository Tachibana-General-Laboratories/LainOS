use gpio::*;

const SYSTMR_LO: Mmio<u32> =        Mmio::new(MMIO_BASE + 0x00003004);
const SYSTMR_HI: Mmio<u32> =        Mmio::new(MMIO_BASE + 0x00003008);

/// Wait N CPU cycles (ARM CPU only)
pub fn wait_cycles(mut n: u32) {
    if n != 0 {
        while { n -= 1; n != 0 } {
            unsafe { asm!("nop" :::: "volatile") }
        }
    }
}

/// Wait N microsec (ARM CPU only)
pub fn wait_msec(n: u32) {
    let mut f: u32;
    let mut t: u32;
    let mut r: u32;

    unsafe {
        // get the current counter frequency
        asm!("mrs $0, cntfrq_el0" : "=r"(f) : : : "volatile");
        // read the current counter
        asm!("mrs $0, cntpct_el0" : "=r"(t) : : : "volatile");
        // calculate expire value for counter
        t += ((f / 1000).wrapping_mul(n)) / 1000;
        while {
            asm!("mrs $0, cntpct_el0" : "=r"(r) : : : "volatile");
            r < t
        } {}
    }
}

/// Get System Timer's counter
pub fn get_system_timer() -> u64 {
    let mut h = 0xFFFF_FFF;
    let mut l;
    // we must read MMIO area as two separate 32 bit reads
    h = SYSTMR_HI.read();
    l = SYSTMR_LO.read();
    // we have to repeat it if high word changed during read
    if h != SYSTMR_HI.read() {
        h = SYSTMR_HI.read();
        l = SYSTMR_LO.read();
    }
    // compose long int value
    ((h as u64) << 32) | l as u64
}

/// Wait N microsec (with BCM System Timer)
pub fn wait_msec_st(n: u32) {
    let t = get_system_timer();
    // we must check if it's non-zero, because qemu does not emulate
    // system timer, and returning constant zero would mean infinite loop
    if t != 0 {
        while { get_system_timer() < t + n as u64 } {}
    }
}
