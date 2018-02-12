use gpio::*;
use core::slice;

const PAGESIZE: isize = 4096;

// granularity
const PT_PAGE: u64 =     0b11;       // 4k granule
const PT_BLOCK: u64 =    0b01;       // 2M granule

// accessibility
const PT_KERNEL: u64 =   0 << 6;      // privileged, supervisor EL1 access only
const PT_USER: u64 =     1 << 6;      // unprivileged, EL0 access allowed
const PT_RW: u64 =       0 << 7;      // read-write
const PT_RO: u64 =       1 << 7;      // read-only
const PT_AF: u64 =       1 << 10;     // accessed flag
const PT_NX: u64 =       1 << 54;     // no execute

// shareability
const PT_OSH: u64 =      2 << 8;      // outter shareable
const PT_ISH: u64 =      3 << 8;      // inner shareable

// defined in MAIR register
const PT_MEM: u64 =      0 << 2;      // normal memory
const PT_DEV: u64 =      1 << 2;      // device MMIO
const PT_NC: u64 =       2 << 2;      // non-cachable

const TTBR_ENABLE: isize = 1;

// get addresses from linker
extern "C" {
    static _data: *mut u8;
    static _end: *mut u8;
}

/// Set up page translation tables and enable virtual memory
pub unsafe fn init() {
    let mut paging = slice::from_raw_parts_mut(_end as *mut u64, 512 * 16);

    // create MMU translation tables at _end

    // TTBR0, identity L1
    paging[0] = (_end.offset(2 * PAGESIZE) as u64) |    // physical address
        PT_PAGE |     // it has the "Present" flag, which must be set, and we have area in it mapped by pages
        PT_AF |       // accessed flag. Without this we're going to have a Data Abort exception
        PT_USER |     // non-privileged
        PT_ISH |      // inner shareable
        PT_MEM;       // normal memory

    // identity L2, first 2M block
    paging[2 * 512]= (_end.offset(3 * PAGESIZE) as u64) | // physical address
        PT_PAGE |     // we have area in it mapped by pages
        PT_AF |       // accessed flag
        PT_USER |     // non-privileged
        PT_ISH |      // inner shareable
        PT_MEM;       // normal memory

    // identity L2 2M blocks
    let b = IO_BASE >> 21;

    // skip 0th, as we're about to map it by L3
    for r in 1..512 {
        paging[2 * 512 + r] = (r<<21) as u64 |  // physical address
            PT_BLOCK |    // map 2M block
            PT_AF |       // accessed flag
            PT_NX |       // no execute
            PT_USER |     // non-privileged
            if r >= b {   // different attributes for device memory
                PT_OSH | PT_DEV
            } else {
                PT_ISH | PT_MEM
            };
    }

    // identity L3
    for r in 0..512 {
        paging[3 * 512 + r as usize] = ((r as u64) * PAGESIZE as u64) |   // physical address
            PT_PAGE |     // map 4k
            PT_AF |       // accessed flag
            PT_USER |     // non-privileged
            PT_ISH |      // inner shareable
            // different for code and data
            if r < 0x80 || r > (_data as u64) / PAGESIZE as u64 {
                PT_RW|PT_NX
            } else {
                PT_RO
            };
    }

    // TTBR1, kernel L1
    paging[512+511] = _end.offset(4 * PAGESIZE) as u64 | // physical address
        PT_PAGE |     // we have area in it mapped by pages
        PT_AF |       // accessed flag
        PT_KERNEL |   // privileged
        PT_ISH |      // inner shareable
        PT_MEM;       // normal memory

    // kernel L2
    paging[4 * 512 + 511]= _end.offset(5 * PAGESIZE) as u64 |   // physical address
        PT_PAGE |     // we have area in it mapped by pages
        PT_AF |       // accessed flag
        PT_KERNEL |   // privileged
        PT_ISH |      // inner shareable
        PT_MEM;       // normal memory

    // kernel L3
    paging[5 * 512] = (IO_BASE + 0x00201000) as u64 |   // physical address
        PT_PAGE |     // map 4k
        PT_AF |       // accessed flag
        PT_NX |       // no execute
        PT_KERNEL |   // privileged
        PT_OSH |      // outter shareable
        PT_DEV;       // device memory

    // okay, now we have to set system registers to enable MMU

    // check for 4k granule and at least 36 bits physical address bus
    let r: u64;
    asm!("mrs $0, id_aa64mmfr0_el1" : "=r" (r) : : : "volatile");

    let b = r & 0xF;
    if r & (0xF<<28) != 0 /*4k*/ || b < 1 /*36 bits*/ {
        println!("ERROR: 4k granule or 36 bit address space not supported");
        return;
    }

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    let r: u64 = (0xFF << 0) |    // AttrIdx=0: normal, IWBWA, OWBWA, NTR
            (0x04 << 8) |    // AttrIdx=1: device, nGnRE (must be OSH too)
            (0x44 <<16);     // AttrIdx=2: non cacheable
    asm! ("msr mair_el1, $0" : : "r" (r) : : "volatile");

    // next, specify mapping characteristics in translate control register
    let r: u64 =
        (0b00 << 37) | // TBI=0, no tagging
        (b    << 32) | // IPS=autodetected
        (0b10 << 30) | // TG1=4k
        (0b11 << 28) | // SH1=3 inner
        (0b01 << 26) | // ORGN1=1 write back
        (0b01 << 24) | // IRGN1=1 write back
        (0b0  << 23) | // EPD1 enable higher half
        (25   << 16) | // T1SZ=25, 3 levels (512G)
        (0b00 << 14) | // TG0=4k
        (0b11 << 12) | // SH0=3 inner
        (0b01 << 10) | // ORGN0=1 write back
        (0b01 <<  8) | // IRGN0=1 write back
        (0b0  <<  7) | // EPD0 enable lower half
        (25   <<  0);  // T0SZ=25, 3 levels (512G)

    asm!("msr tcr_el1, $0;\n isb" : : "r" (r) : : "volatile");

    // tell the MMU where our translation tables are.
    // TTBR_ENABLE bit not documented, but required
    {
        // lower half, user space
        let addr = _end.offset(TTBR_ENABLE) as u64;
        asm!("msr ttbr0_el1, $0" : : "r" (addr) : : "volatile");
        // upper half, kernel space
        let addr = _end.offset(TTBR_ENABLE + PAGESIZE) as u64;
        asm!("msr ttbr1_el1, $0" : : "r" (addr));
    }

    // finally, toggle some bits in system control register to enable page translation
    let mut r: u64;
    asm!("dsb ish;\n isb;\n mrs $0, sctlr_el1" : "=r" (r) : : : "volatile");

    r |= 0xC00800;     // set mandatory reserved bits

    r &= !((1<<25) |   // clear EE, little endian translation tables
           (1<<24) |   // clear E0E
           (1<<19) |   // clear WXN
           (1<<12) |   // clear I, no instruction cache
           (1<< 4) |   // clear SA0
           (1<< 3) |   // clear SA
           (1<< 2) |   // clear C, no cache at all
           (1<< 1));   // clear A, no aligment check

    r |=   (1<<0);     // set M, enable MMU

    asm!("msr sctlr_el1, $0;\n isb" : : "r" (r) : : "volatile");
}
