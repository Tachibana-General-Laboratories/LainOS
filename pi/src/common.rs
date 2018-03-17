/// The address where I/O peripherals are mapped to.
pub const IO_BASE: usize = 0x3F000000;

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


#[inline(always)]
pub fn spin_wait<F>(mut f: F)
    where F: FnMut() -> bool
{
    while {
        unsafe { asm!("nop" :::: "volatile"); }
        f()
    } {}
}
