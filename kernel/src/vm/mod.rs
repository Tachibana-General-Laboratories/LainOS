mod address;
mod entry;
mod table;
mod huge;
mod page;

use alloc::boxed::Box;
use alloc::vec::Vec;

use sys::volatile::prelude::*;

use pi::common::IO_BASE as _IO_BASE;
use core::fmt;

use aarch64;

pub use self::entry::Entry;
pub use self::table::{Table, Level, L0, L1, L2, L3};
pub use self::address::{PhysicalAddr, VirtualAddr};

pub use self::huge::{Huge, HUGESZ};
pub use self::page::{Page, PAGESZ};

pub const UPPER_SPACE_MASK: usize = 0xFFFFFF80_00000000;
pub const LOWER_SPACE_MASK: usize = 0x0000007F_FFFFFFFF;

const IO_BASE: usize = _IO_BASE & LOWER_SPACE_MASK;

use allocator::util::{align_up, align_down};
use allocator::safe_box;

// get addresses from linker
extern "C" {
    static mut _start: u8;
    static mut _data: u8;
    static mut _end: u8;
}

pub fn kernel_start() -> PhysicalAddr {
    let p = unsafe { &mut _start as *mut _ };
    (((p as usize) & LOWER_SPACE_MASK) as *mut u8).into()
}

pub fn kernel_data() -> PhysicalAddr {
    let p = unsafe { &mut _data as *mut _ };
    (((p as usize) & LOWER_SPACE_MASK) as *mut u8).into()
}

pub fn kernel_end() -> PhysicalAddr {
    let p = unsafe { &mut _end as *mut _ };
    (((p as usize) & LOWER_SPACE_MASK) as *mut u8).into()
}

#[inline(always)]
pub fn v2p(v: VirtualAddr) -> Option<PhysicalAddr> {
    v.as_usize().checked_sub(UPPER_SPACE_MASK).map(Into::into)
}

#[inline(always)]
pub fn p2v(p: PhysicalAddr) -> VirtualAddr {
    ((p.as_usize() | UPPER_SPACE_MASK) as *mut u8).into()
}

pub struct Memory {
    root: L1,
    areas: Vec<Area>,
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Memory{:?}", self.areas)
    }
}

impl Memory {
    pub fn new() -> Option<Box<Self>> {
        Some(safe_box(Self {
            root: Page::new_zeroed()?.into(),
            areas: Vec::new(),
        })?)
    }

    pub fn ttbr(&mut self) -> PhysicalAddr {
        self.root.physical()
    }

    #[must_use]
    pub fn area_rx(&mut self, area: Area) -> Option<()> {
        self.area(area.prot(Prot::RX))
    }

    #[must_use]
    pub fn area_rw(&mut self, area: Area) -> Option<()> {
        self.area(area.prot(Prot::RW))
    }

    #[must_use]
    pub fn area_ro(&mut self, area: Area) -> Option<()> {
        self.area(area.prot(Prot::RO))
    }

    #[must_use]
    pub fn area_dev(&mut self, area: Area) -> Option<()> {
        self.area(area.entry(Entry::USER_DEV))
    }

    #[must_use]
    pub fn area(&mut self, area: Area) -> Option<()> {
        /*
        use console::kprintln;
        match area.physical {
            Some(p) => kprintln!("area: {}-{} to {}", area.start, area.end, p),
            None =>    kprintln!("area: {}-{}", area.start, area.end),
        }
        */

        let mut addr = align_down(area.start.as_usize(), PAGESZ);
        let end = align_up(area.end.as_usize(), PAGESZ);
        let entry = area.entry;
        let p = area.physical;

        self.areas.try_reserve(1).ok()?;
        self.areas.push(area);

        if let Some(p) = p {
            let mut p = align_down(p.as_usize(), PAGESZ);
            while addr < end {
                let v = (addr as *mut u8).into();
                let l2 = self.root.next_table_or(v, Entry::USER_BASE)?;
                if addr % HUGESZ == 0 && (end - addr) >= HUGESZ {
                    l2[v].write(Entry::block(p.into()) | entry);
                    addr += HUGESZ;
                    p += HUGESZ;
                } else {
                    let l3 = l2.next_table_or(v, Entry::USER_BASE)?;
                    l3[v].write(Entry::page(p.into()) | entry);
                    addr += PAGESZ;
                    p += PAGESZ;
                }
            }
        }

        Some(())
    }

    pub fn find_area_mut(&mut self, addr: VirtualAddr) -> Option<&mut Area> {
        self.areas.iter_mut().find(|area| area.contains(addr))
    }

    pub fn find_area(&self, addr: VirtualAddr) -> Option<&Area> {
        self.areas.iter().find(|area| area.contains(addr))
    }

    pub fn has_addr(&self, addr: VirtualAddr) -> bool {
        self.areas.iter().any(|area| area.contains(addr))
    }

    pub fn page_fault(&mut self, addr: u64) {
        let addr = VirtualAddr::from((addr as usize) as *mut u8);
        let page = Page::new().expect("allocate page");
        unsafe { self.add_page(addr, page).expect("add page") };
    }

    unsafe fn add_page(&mut self, v: VirtualAddr, p: Page) -> Option<()> {
        let entry = self.find_area(v)?.entry;
        let l2 = self.root.next_table_or(v, Entry::USER_BASE)?;
        let l3 = l2.next_table_or(v, Entry::USER_BASE)?;
        l3[v].write(Entry::page(p.into()) | entry);
        Some(())
    }
}

bitflags! {
    pub struct Prot: u8 {
        const READ  = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC  = 1 << 2;

        const RO = Self::READ.bits;
        const RW = Self::READ.bits | Self::WRITE.bits;
        const RX = Self::READ.bits | Self::EXEC.bits;
    }
}

bitflags! {
    pub struct Map: u8 {
        const EXEC  = 1 << 0;
        const WRITE = 1 << 1;
        const READ  = 1 << 2;

        const CAN_EXEC  = 1 << 4;
        const CAN_WRITE = 1 << 5;
        const CAN_READ  = 1 << 6;

        const ANON      = 1 << 10;
        const FIXED     = 1 << 11;
        const PRIVATE   = 1 << 12;
        const SHARED    = 1 << 13;
        const STACK     = 1 << 14;
    }
}

#[derive(Debug)]
pub struct Area {
    pub start: VirtualAddr,
    pub end: VirtualAddr,
    pub entry: Entry,
    pub physical: Option<PhysicalAddr>,
}

impl Area {
    pub fn new(start: usize, end: usize) -> Self {
        let start = VirtualAddr::from(start as *mut u8);
        let end = VirtualAddr::from(end as *mut u8);
        Self {
            start, end,
            entry: Entry::INVALID,
            physical: None,
        }
    }
    pub fn entry(mut self, entry: Entry) -> Self {
        self.entry = entry;
        self
    }
    pub fn prot(mut self, p: Prot) -> Self {
        self.entry = match p {
            Prot::RO => Entry::USER_RO,
            Prot::RW => Entry::USER_RW,
            Prot::RX => Entry::USER_RX,
            _ => unimplemented!(),
        };
        self
    }
    pub fn map_to(mut self, p: PhysicalAddr) -> Self {
        self.physical = Some(p);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    pub fn contains(&self, addr: VirtualAddr) -> bool {
        self.start <= addr && addr < self.end
    }

    pub fn intersects(&self, other: Self) -> bool {
        self.contains(other.start) || self.contains(other.end)
    }
}

/// Set up page translation tables and enable virtual memory
pub fn initialize() {
    #![allow(non_snake_case)]

    static mut L1: Table<L1> = Table::empty();
    static mut L2: Table<L2> = Table::empty();
    static mut L3: Table<L3> = Table::empty();

    let NORMAL: Entry = Entry::AF | Entry::ISH;
    let DATA: Entry  = NORMAL | Entry::XN | Entry::PXN;
    let CODE: Entry  = NORMAL | Entry::AP_RO;
    let DEV: Entry   = Entry::AF | Entry::OSH | Entry::XN | Entry::ATTR_1;

    let start = kernel_start().as_usize();
    let data = kernel_data().as_usize();

    let ttbr = unsafe {
        L1.get_mut(0).write(Entry::table((&mut L2 as *mut Table<L2>).into()) | NORMAL);
        L2.get_mut(0).write(Entry::table((&mut L3 as *mut Table<L3>).into()) | NORMAL);
        for n in 1..512usize {
            let addr = n * HUGESZ;
            L2.get_mut(n).write(Entry::block(addr.into()) | if addr < IO_BASE { DATA } else { DEV });
        }
        for n in 0..512usize {
            let addr = n * PAGESZ;
            L3.get_mut(n).write(Entry::page(addr.into()) | if addr < start || addr >= data { DATA } else { CODE });
        }

        &mut L1 as *mut _ as u64
    };

    aarch64::set_ttbr0_el1(0, ttbr, true);
    aarch64::set_ttbr1_el1(1, ttbr, true);

    // okay, now we have to set system registers to enable MMU

    // check for 4k granule and at least 36 bits physical address bus
    let mmfr = aarch64::MMFR::new();

    let par = mmfr.physical_address_range();
    assert!(mmfr.support_4kb_granulate(), "4k granulate not supported");
    assert!(par >= aarch64::PhysicalAddressRange::R36, "36 bit address space not supported");

    let ips = par.raw() as u64;

    // first, set Memory Attributes array,
    // indexed by PT_MEM, PT_DEV, PT_NC in our example
    let mair: u64 =
        (0xFF << 0) |   // AttrIdx=0: normal, IWBWA, OWBWA, NTR
        (0x04 << 8) |   // AttrIdx=1: device, nGnRE (must be OSH too)
        (0x44 <<16);    // AttrIdx=2: non cacheable

    // next, specify mapping characteristics in translate control register
    let tcr: u64 =
        (0b00 << 37) | // TBI=0, no tagging
        (ips  << 32) | // IPS=autodetected
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

    unsafe {
        asm!("
            msr mair_el1, $0
            msr tcr_el1, $1
            isb
            "
            :: "r"(mair), "r"(tcr)
            :: "volatile");
    }

    // finally, toggle some bits in system control register to enable page translation
    unsafe {
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
}
