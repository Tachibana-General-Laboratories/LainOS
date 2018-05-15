/// The address where I/O peripherals are mapped to.
//pub const IO_BASE: usize = 0x3F000000;
pub const IO_BASE: usize = 0xFFFF_FF80_3F000000;

/// Generates `pub enums` with no variants for each `ident` passed in.
pub macro states($($name:ident),*) {
    $(
        /// A possible state.
        #[doc(hidden)]
        pub enum $name {  }
    )*
}

#[inline(always)]
pub fn spin_sleep_cycles(cycles: u32) {
    for _ in 0..cycles {
        unsafe { asm!("nop" :::: "volatile") }
    }
}

/// Wait N microsec (ARM CPU only)
#[inline(always)]
pub fn spin_sleep_us(n: u32) {
    let f: u32;
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


#[inline(always)]
pub fn spin_wait<F>(mut f: F)
    where F: FnMut() -> bool
{
    while {
        unsafe { asm!("nop" :::: "volatile"); }
        f()
    } {}
}
