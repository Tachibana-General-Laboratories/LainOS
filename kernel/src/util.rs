use console::{kprint, kprintln};

#[inline(always)]
pub fn nop_while<F>(mut f: F)
    where F: FnMut() -> bool
{
    while {
        unsafe { asm!("nop" :::: "volatile"); }
        f()
    } {}
}

/// Wait N CPU cycles (ARM CPU only)
#[inline(always)]
pub fn wait_cycles(mut n: u32) {
    if n != 0 {
        while { n -= 1; n != 0 } {
            unsafe { asm!("nop" :::: "volatile") }
        }
    }
}

/// Wait N microsec (ARM CPU only)
#[inline(always)]
pub fn wait_msec(n: u32) {
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

pub fn dump(mut ptr: *const u8, size: usize) {
    let count = size / 16 + (size % 16 != 0) as usize;
    for _ in 0..count {
        unsafe {
            kprint!("{:08X}  ", ptr as u32);
            for i in 0..16 {
                kprint!("{:02X} ", *ptr.offset(i));
                if i == 7 {
                    kprint!(" ");
                }
            }
            kprint!(" ");
            for i in 0..16 {
                match *ptr.offset(i) {
                    c @ 32...127 => kprint!("{}", c as char),
                    _ => kprint!("."),
                }
            }
            kprintln!("");
            ptr = ptr.offset(16);
        }
    }
}
