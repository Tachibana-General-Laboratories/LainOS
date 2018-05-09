use pi::*;
use pi::common::IO_BASE;
use core::slice;
use sys::volatile::prelude::*;
use sys::volatile::Volatile;

use aarch64;

use vm::{Entry, PhysicalAddr};

use console::kprintln;

pub const PAGESIZE_4K: usize = 2 << 11;
pub const PAGESIZE_16K: usize = 2 << 13;
pub const PAGESIZE_64K: usize = 2 << 15;


pub const PAGESIZE: usize = 4096;
const TTBR_ENABLE: u64 = 1;

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

// get addresses from linker
extern "C" {
    static _data: usize;
    static _ttbr_start: usize;
}

struct Table<'a> {
    l1: &'a mut [Volatile<Entry>],
    l2: &'a mut [Volatile<Entry>],
    l3: &'a mut [Volatile<Entry>],
}

impl<'a> Table<'a> {
    unsafe fn new(l1: usize, l2: usize, l3: usize) -> Self {
        let l1 = slice::from_raw_parts_mut(l1 as *mut Volatile<Entry>, 512);
        let l2 = slice::from_raw_parts_mut(l2 as *mut Volatile<Entry>, 512);
        let l3 = slice::from_raw_parts_mut(l3 as *mut Volatile<Entry>, 512);
        Self { l1, l2, l3 }
    }
    /*
    fn write123dev(&mut self, virt: usize, phy: usize) {
        //  l0 = ((virt >> 39) & 0b111111111) as usize;
        let l1 = ((virt >> 30) & 0b111111111) as usize;
        let l2 = ((virt >> 21) & 0b111111111) as usize;
        let l3 = ((virt >> 12) & 0b111111111) as usize;

        kprintln!("[{} {} {}] p{:X} -> v{:X}", l1, l2, l3, phy, virt);

        const BASE_FLAGS: u64 = PT_PAGE | PT_AF | PT_XN | PT_KERNEL;

        // TTBR1, kernel L1
        self.l1[l1].write(self.l2.as_ptr() as u64 | BASE_FLAGS | PT_ISH | PT_MEM);
        self.l2[l2].write(self.l3.as_ptr() as u64 | BASE_FLAGS | PT_ISH | PT_MEM);
        self.l3[l3].write(phy as u64 | BASE_FLAGS | PT_OSH | PT_DEV);
    }
    */
}

/// Set up page translation tables and enable virtual memory
pub extern fn init_mmu() {
    // create MMU translation tables at _end

    // user space
    let ttbr0 = unsafe {
        let l1 = _ttbr_start + PAGESIZE * 0;
        let l2 = _ttbr_start + PAGESIZE * 2;
        let l3 = _ttbr_start + PAGESIZE * 3;
        Table::new(l1, l2, l3)
    };

    // kernel space
    let ttbr1 = unsafe {
        let l1 = _ttbr_start + PAGESIZE * 1;
        let l2 = _ttbr_start + PAGESIZE * 4;
        let l3 = _ttbr_start + PAGESIZE * 5;
        Table::new(l1, l2, l3)
    };

    // identity L1
    let addr = PhysicalAddr::from(ttbr0.l2.as_mut_ptr());
    ttbr0.l1[0].write(Entry::table(addr) | Entry::AF | Entry::SH_INNER | Entry::AP_EL0);
    let addr = PhysicalAddr::from(ttbr1.l2.as_mut_ptr());
    ttbr1.l1[1].write(Entry::table(addr) | Entry::AF | Entry::SH_INNER);

    // identity L2, first 2M block
    let addr = PhysicalAddr::from(ttbr0.l3.as_mut_ptr());
    ttbr0.l2[0].write(Entry::table(addr) | Entry::AF | Entry::SH_INNER | Entry::AP_EL0);

    // identity L2 2M blocks
    let b = IO_BASE >> 21;

    // skip 0th, as we're about to map it by L3
    for r in 1..512usize {
        let addr = PhysicalAddr::from(((r<<21) as u64) as *mut u8);

        // different attributes for device memory
        let attributes = if r >= b {
            Entry::SH_OUTER.with_attr_index(1)
        } else {
            Entry::SH_INNER
        };

        let block = Entry::block(addr) | Entry::AF | Entry::XN | attributes;
        ttbr0.l2[r].write(block | Entry::AP_EL0);
        ttbr1.l2[r].write(block);
    }

    // identity L3
    for r in 0..512usize {
        let addr = PhysicalAddr::from((r * PAGESIZE) as *mut u8);

        // different for code and data
        let attributes = if r < 0x80 || r as u64 > unsafe { _data as u64 } / PAGESIZE as u64 {
            Entry::XN
        } else {
            Entry::AP_RO
        };

        let page = Entry::page(addr) | Entry::AF | Entry::SH_INNER | attributes;
        ttbr0.l3[r].write(page | Entry::AP_EL0);
        ttbr1.l3[r].write(page);
    }

    /*
    if true {
        let addr = IO_BASE + 0x00201000;
        ttbr1.write123dev(addr as usize | 0xFFFF_FF80_0000_0000, addr);
    }
    */

    unsafe {
        let ttbr0 = _ttbr_start as *mut u8;
        let ttbr1 = (_ttbr_start + PAGESIZE) as *mut u8;

        enable_mmu(PhysicalAddr::from(ttbr0), PhysicalAddr::from(ttbr1));
    }
}

unsafe fn enable_mmu(ttbr0: PhysicalAddr, ttbr1: PhysicalAddr) {
    // okay, now we have to set system registers to enable MMU

    // check for 4k granule and at least 36 bits physical address bus
    let mmfr = aarch64::MMFR::new();

    let b = mmfr.physical_address_range();
    if !mmfr.support_4kb_granulate() || b < aarch64::PhysicalAddressRange::R36 {
        panic!("4k granule or 36 bit address space not supported");
    }
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
        (0b0  << 23) | // EPD1 enable higher half
        (25   << 16) | // T1SZ=25, 3 levels (512G)
        (0b00 << 14) | // TG0=4k
        (0b11 << 12) | // SH0=3 inner
        (0b01 << 10) | // ORGN0=1 write back
        (0b01 <<  8) | // IRGN0=1 write back
        (0b0  <<  7) | // EPD0 enable lower half
        (25   <<  0);  // T0SZ=25, 3 levels (512G)

    asm!("msr tcr_el1, $0; isb" : : "r" (r) : : "volatile");

    // tell the MMU where our translation tables are.
    // TTBR_ENABLE bit not documented, but required
    {
        // lower half, user space
        let addr = ttbr0.as_u64() + TTBR_ENABLE;
        asm!("msr ttbr0_el1, $0" :: "r"(addr) :: "volatile");
        // upper half, kernel space
        let addr = ttbr1.as_u64() + TTBR_ENABLE;
        asm!("msr ttbr1_el1, $0" :: "r"(addr) :: "volatile");
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
