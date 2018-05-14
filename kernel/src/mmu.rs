use pi::*;
use pi::common::IO_BASE;
use core::{slice, mem};
use sys::volatile::Volatile;
use sys::volatile::prelude::*;

use aarch64;
use vm::*;
use console::kprintln;

const TTBR_ENABLE: u64 = 1;

/*
// granularity
pub const PT_PAGE: u64 =     0b11;       // 4k granule
pub const PT_BLOCK: u64 =    0b01;       // 2M granule

// accessibility
pub const PT_KERNEL: u64 =   0 << 6;      // privileged, supervisor EL1 access only
pub const PT_USER: u64 =     1 << 6;      // unprivileged, EL0 access allowed
pub const PT_RW: u64 =       0 << 7;      // read-write
pub const PT_RO: u64 =       1 << 7;      // read-only
pub const PT_AF: u64 =       1 << 10;     // accessed flag
pub const PT_XN: u64 =       1 << 54;     // no execute

// shareability
pub const PT_OSH: u64 =      2 << 8;      // outter shareable
pub const PT_ISH: u64 =      3 << 8;      // inner shareable

// defined in MAIR register
pub const PT_MEM: u64 =      0 << 2;      // normal memory
pub const PT_DEV: u64 =      1 << 2;      // device MMIO
pub const PT_NC: u64 =       2 << 2;      // non-cachable
*/

// get addresses from linker
extern "C" {
    static _start: u8;
    static _data: u8;
}

/// Set up page translation tables and enable virtual memory
pub fn init_mmu() {
    let INNER: Entry = Entry::AF | Entry::AP_EL0 | Entry::SH_INNER;

    let DATA: Entry = INNER | Entry::XN | Entry::PXN;
    let CODE: Entry = INNER | Entry::AP_RO;
    let DEV: Entry = Entry::AF | Entry::AP_EL0 | Entry::SH_OUTER | Entry::XN.with_attr_index(1);

    let start = unsafe { &_start as *const _ as usize };
    let data = unsafe { &_data as *const _ as usize };

    kprintln!("start: {:x}, data: {:x}", start, data);

    // create MMU translation tables
    let mut map = Memory::new().unwrap()
        // different for code and data
        .area(0, start).attrs(INNER, INNER, DATA).map()
        .area(start, data).attrs(INNER, INNER, CODE).map()
        .area(data, IO_BASE).attrs(INNER, DATA, DATA).map()
        // different attributes for device memory
        .area(IO_BASE, 512 << 21).attrs(INNER, DEV, DEV).map();

    unsafe {
        enable_mmu(map.ttbr());
    }
    mem::forget(map);
}

unsafe fn enable_mmu(ttbr: PhysicalAddr) {
    // okay, now we have to set system registers to enable MMU

    // check for 4k granule and at least 36 bits physical address bus
    let mmfr = aarch64::MMFR::new();

    let b = mmfr.physical_address_range();
    assert!(mmfr.support_4kb_granulate(), "4k granulate not supported");
    assert!(b >= aarch64::PhysicalAddressRange::R36, "36 bit address space not supported");

    let b = b.raw() as u64;

    // first, set Memory Attributes array, indexed by PT_MEM, PT_DEV, PT_NC in our example
    let r: u64 = (0xFF << 0) |    // AttrIdx=0: normal, IWBWA, OWBWA, NTR
                 (0x04 << 8) |    // AttrIdx=1: device, nGnRE (must be OSH too)
                 (0x44 <<16);     // AttrIdx=2: non cacheable
    asm!("msr mair_el1, $0" :: "r"(r) :: "volatile");

    // next, specify mapping characteristics in translate control register
    let r: u64 =
        (0b00 << 37) | // TBI=0, no tagging
        (b    << 32) | // IPS=autodetected
        (0b10 << 30) | // TG1=4k
        (0b11 << 28) | // SH1=3 inner
        (0b01 << 26) | // ORGN1=1 write back
        (0b01 << 24) | // IRGN1=1 write back
        //(0b0  << 23) | // EPD1 enable higher half
        (0b1  << 23) | // EPD1 disable higher half
        (25   << 16) | // T1SZ=25, 3 levels (512G)
        (0b00 << 14) | // TG0=4k
        (0b11 << 12) | // SH0=3 inner
        (0b01 << 10) | // ORGN0=1 write back
        (0b01 <<  8) | // IRGN0=1 write back
        (0b0  <<  7) | // EPD0 enable lower half
        (25   <<  0);  // T0SZ=25, 3 levels (512G)

    asm!("msr tcr_el1, $0; isb" :: "r"(r) :: "volatile");

    // tell the MMU where our translation tables are.
    // TTBR_ENABLE bit not documented, but required
    {
        // lower half, user space
        let addr = ttbr.as_u64() + TTBR_ENABLE;
        asm!("msr ttbr0_el1, $0" :: "r"(addr) :: "volatile");
        // upper half, kernel space
        //let addr = ttbr1.as_u64() + TTBR_ENABLE;
        //asm!("msr ttbr1_el1, $0" :: "r"(addr) :: "volatile");
    }

    // finally, toggle some bits in system control register to enable page translation
    let mut r: u64;
    asm!("dsb ish; isb; mrs $0, sctlr_el1" : "=r"(r) : : : "volatile");

    r |= 0xC00800;      // set mandatory reserved bits
    r &= !((1<<25)  |   // clear EE, little endian translation tables
           (1<<24)  |   // clear E0E
           (1<<19)  |   // clear WXN
           (1<<12)  |   // clear I, no instruction cache
           (1<< 4)  |   // clear SA0
           (1<< 3)  |   // clear SA
           (1<< 2)  |   // clear C, no cache at all
           (1<< 1));    // clear A, no aligment check
    r |=    1<< 0;      // set M, enable MMU

    asm!("msr sctlr_el1, $0; isb" :: "r"(r) :: "volatile");
}
