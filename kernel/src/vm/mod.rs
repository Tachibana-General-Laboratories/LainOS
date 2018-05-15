mod address;
mod entry;
mod table;

use alloc::boxed::Box;
use alloc::vec::Vec;

use sys::volatile::prelude::*;

use pi::common::IO_BASE as _IO_BASE;
use core::{fmt, mem};

use aarch64;

pub use self::entry::Entry;
pub use self::table::{Table, Level, Level0, Level1, Level2, Level3};
pub use self::address::{PhysicalAddr, VirtualAddr};

pub const PAGESZ: usize = 4096;
pub const HUGESZ: usize = 4096 * 512;
pub const KERNEL_SPACE_MAKS: usize = 0xFFFFFF80_00000000;
pub const USER_SPACE_MASK: usize   = 0x0000007F_FFFFFFFF;
pub const PHYSICAL_SPACE_MASK: usize   = 0x0000007F_FFFFFFFF;

pub const IO_BASE: usize = _IO_BASE & PHYSICAL_SPACE_MASK;

use ALLOCATOR;
use alloc::allocator::{Alloc, Layout};
use allocator::util::{align_up, align_down};

pub fn kernel_into_physical<T: ?Sized>(v: *mut T) -> PhysicalAddr {
    let v = (v as *mut () as usize & PHYSICAL_SPACE_MASK) as *mut u8;
    PhysicalAddr::from(v)
}

pub unsafe fn alloc_table<'a, L: Level>() -> Option<(&'a mut Table<L>, PhysicalAddr)> {
    let layout = Layout::from_size_align_unchecked(PAGESZ, PAGESZ);
    let mem = (&ALLOCATOR).alloc_zeroed(layout).ok()?;
    let table: &mut Table<L> = &mut *(mem.cast().as_ptr());
    Some((table, kernel_into_physical(mem.as_ptr())))
}

pub unsafe fn alloc_page() -> Option<PhysicalAddr> {
    let layout = Layout::from_size_align_unchecked(PAGESZ, PAGESZ);
    let mem = (&ALLOCATOR).alloc(layout).ok()?;
    Some(kernel_into_physical(mem.cast::<usize>().as_ptr().into()))
}

pub unsafe fn alloc_huge() -> Option<PhysicalAddr> {
    let layout = Layout::from_size_align_unchecked(HUGESZ, HUGESZ);
    let mem = (&ALLOCATOR).alloc(layout).ok()?;
    Some(kernel_into_physical(mem.cast::<usize>().as_ptr().into()))
}

pub struct Memory {
    pub root: Table<Level1>,
    pub areas: Vec<Area>,
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Memory{:?}", self.areas)
    }
}


impl Memory {
    pub fn new() -> Self {
        Self {
            root: Table::empty(),
            areas: Vec::new(),
        }
    }

    pub fn ttbr(&mut self) -> PhysicalAddr {
        kernel_into_physical(&mut self.root as *mut Table<Level1>)
    }

    pub fn area(self, start: usize, end: usize) -> AreaBuilder {
        AreaBuilder::new(self, start, end)
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

    unsafe fn alloc_huge(&mut self, v: VirtualAddr) -> Option<PhysicalAddr> {
        let p = alloc_huge()?;
        self.map_huge(v, p)?;
        Some(p)
    }

    unsafe fn alloc_page(&mut self, v: VirtualAddr) -> Option<PhysicalAddr> {
        let p = alloc_page()?;
        self.map_page(v, p)?;
        Some(p)
    }

    unsafe fn map_huge(&mut self, v: VirtualAddr, p: PhysicalAddr) -> Option<()> {
        let (a1, a2, _) = self.find_area(v)?.entries();
        let l2 = self.root.next_table_or(v, || Some((a1, alloc_table()?)))?;
        l2[v].write(Entry::block(p) | a2);
        Some(())
    }

    unsafe fn map_page(&mut self, v: VirtualAddr, p: PhysicalAddr) -> Option<()> {
        let (a1, a2, a3) = self.find_area(v)?.entries();
        let l1 = &mut self.root;
        let l2 = l1.next_table_or(v, || Some((a1, alloc_table()?)))?;
        let l3 = l2.next_table_or(v, || Some((a2, alloc_table()?)))?;
        l3[v].write(Entry::page(p) | a3);
        Some(())
    }
}

/*
impl Drop for Memory {
    fn drop(&mut self) {
        for entry in self.root.iter().filter(|e| e.is_valid()) {
            if let Some(l2) = unsafe { entry.as_table::<Level2>() } {
                for entry in l2.iter().filter(|e| e.is_valid()) {
                    if let Some(l3) = unsafe { entry.as_table::<Level3>() } {
                        for entry in l3.iter().filter(|e| e.is_valid()) {
                            drop(unsafe {
                                Box::from_raw(entry.addr().as_mut_ptr() as *mut [u8; PAGESZ])
                            })
                        }
                    } else {
                        unimplemented!("L2 block")
                    }
                }
            } else {
                unimplemented!("L1 block")
            }
        }
    }
}
*/


/*
impl<L: Level> Drop for Table<L> {
    fn drop(&mut self) {
        for entry in self.entries.iter().filter(|e| e.need_drop()) {
            if mem::size_of::<L::Block>() == 0 {
                drop(unsafe {
                    Box::from_raw(entry.addr().as_mut_ptr() as *mut [u8; PAGESZ])
                })
            } else if entry.is_block() {
                drop(unsafe {
                    let addr = entry.addr().as_mut_ptr();
                    let addr = addr as *mut L::Block;
                    Box::from_raw(addr)
                })
            } else {
                drop(unsafe {
                    let addr = entry.addr().as_mut_ptr();
                    let addr = addr as *mut Table<L::NextLevel>;
                    Box::from_raw(addr)
                })
            }
        }
    }
}
*/

pub struct AreaBuilder {
    mem: Memory,
    area: Area,
}

impl AreaBuilder {
    fn new(mem: Memory, start: usize, end: usize) -> Self {
        let start = VirtualAddr::from(start);
        let end = VirtualAddr::from(end);
        Self {
            mem, area: Area {
                start, end,
                l1: Entry::INVALID,
                l2: Entry::INVALID,
                l3: Entry::INVALID,
            },
        }
    }

    pub fn attrs(mut self, l1: Entry, l2: Entry, l3: Entry) -> Self {
        self.area.l1 = l1;
        self.area.l2 = l2;
        self.area.l3 = l3;
        self
    }

    pub fn l1(mut self, e: Entry) -> Self { self.area.l1 = e; self }
    pub fn l2(mut self, e: Entry) -> Self { self.area.l2 = e; self }
    pub fn l3(mut self, e: Entry) -> Self { self.area.l3 = e; self }

    pub fn create(self) -> Memory {
        let Self { mut mem, area } = self;
        mem.areas.push(area);
        mem
    }

    pub fn alloc(self) -> Memory {
        let Self { mut mem, area } = self;
        let mut addr = area.start.as_usize();
        let end = area.end.as_usize();
        mem.areas.push(area);
        while addr < end {
            let v = VirtualAddr::from(addr);
            if addr % HUGESZ == 0 && (end - addr) >= HUGESZ {
                unsafe { mem.alloc_huge(v).expect("alloc_huge") };
                addr += HUGESZ;
                continue;
            }
            unsafe { mem.alloc_page(v).expect("alloc_page") };
            addr += PAGESZ;
        }
        mem
    }

    pub fn map(self) -> Memory {
        let Self { mut mem, area } = self;
        let mut addr = area.start.as_usize();
        let end = area.end.as_usize();
        mem.areas.push(area);
        while addr < end {
            if addr % HUGESZ == 0 && (end - addr) >= HUGESZ {
                unsafe { mem.map_huge(addr.into(), addr.into()); }
                addr += HUGESZ;
            } else {
                unsafe { mem.map_page(addr.into(), addr.into()); }
                addr += PAGESZ;
            }
        }
        mem
    }

    pub fn map_to(self, p: PhysicalAddr) -> Memory {
        let mut p = p.as_usize();

        let Self { mut mem, area } = self;
        //let mut addr = align_down(area.start.as_usize(), PAGESZ);
        //let end = align_up(area.end.as_usize(), PAGESZ);
        let mut addr = area.start.as_usize();
        let end = area.end.as_usize();
        mem.areas.push(area);
        while addr < end {
            if addr % HUGESZ == 0 && (end - addr) >= HUGESZ {
                unsafe { mem.map_huge(addr.into(), p.into()); }
                addr += HUGESZ;
                p += HUGESZ;
                continue;
            }
            unsafe { mem.map_page(addr.into(), p.into()); }
            addr += PAGESZ;
            p += PAGESZ;
        }
        mem
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
    pub l1: Entry,
    pub l2: Entry,
    pub l3: Entry,
}

impl Area {
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    pub fn contains(&self, addr: VirtualAddr) -> bool {
        self.start <= addr && addr < self.end
    }

    pub fn intersects(&self, other: Self) -> bool {
        self.contains(other.start) || self.contains(other.end)
    }

    fn entries(&self) -> (Entry, Entry, Entry) {
        (self.l1, self.l2, self.l3)
    }

    /*
    pub fn is_stack(&self) -> bool {
        self.flags.contains(Map::STACK)
    }
    */
}

// get addresses from linker
extern "C" {
    static mut _start: u8;
    static mut _data: u8;
    static mut _end: u8;
}

pub fn kernel_start() -> PhysicalAddr {
    unsafe { kernel_into_physical(&mut _start as *mut _) }
}

pub fn kernel_data() -> PhysicalAddr {
    unsafe { kernel_into_physical(&mut _data as *mut _) }
}

pub fn kernel_end() -> PhysicalAddr {
    unsafe { kernel_into_physical(&mut _end as *mut _) }
}

pub fn initialize() {
    #![allow(non_snake_case)]

    let start = kernel_start().as_usize();
    let data = kernel_data().as_usize();


    // create MMU translation tables

    // lower half, user space
    let INNER: Entry = Entry::AF | Entry::AP_EL0 | Entry::SH_INNER;
    let DATA: Entry  = Entry::AF | Entry::AP_EL0 | Entry::SH_INNER | Entry::XN | Entry::PXN;
    let CODE: Entry  = Entry::AF | Entry::AP_EL0 | Entry::SH_INNER | Entry::AP_RO;
    let DEV: Entry   = Entry::AF | Entry::AP_EL0 | Entry::SH_OUTER | Entry::XN | Entry::ATTR_1;

    // create MMU translation tables
    let mut map = Memory::new()
        // different for code and data
        .area(0, start).attrs(INNER, INNER, DATA).map()
        .area(start, data).attrs(INNER, INNER, CODE).map()
        .area(data, IO_BASE).attrs(INNER, DATA, DATA).map()
        // different attributes for device memory
        .area(IO_BASE, 512 << 21).attrs(INNER, DEV, DEV).map()
    ;

    //::console::kprintln!("start: {:x} data: {:x} iobase: {:x} ttbr: {}", start, data, IO_BASE, map.ttbr());

    aarch64::flush_user_tlb();

    // tell the MMU where our translation tables are.
    aarch64::set_ttbr0_el1(1, map.ttbr().as_u64(), true);
    mem::forget(Box::new(map));
}

static mut L1: Table<Level1> = Table::empty();
static mut L2: Table<Level2> = Table::empty();
static mut L3: Table<Level3> = Table::empty();

/// Set up page translation tables and enable virtual memory
#[inline(never)]
pub fn enable_mmu() {
    #![allow(non_snake_case)]

    let INNER: Entry = Entry::AF | Entry::SH_INNER;
    let DATA: Entry  = Entry::AF | Entry::SH_INNER | Entry::XN | Entry::PXN;
    let CODE: Entry  = Entry::AF | Entry::SH_INNER | Entry::AP_RO;
    let DEV: Entry   = Entry::AF | Entry::SH_OUTER | Entry::XN | Entry::ATTR_1;

    let start = kernel_start().as_usize();
    let data = kernel_data().as_usize();

    let ttbr = unsafe {
        L1.get_mut(0).write(Entry::table((&mut L2 as *mut _).into()) | INNER);
        L2.get_mut(0).write(Entry::table((&mut L3 as *mut _).into()) | INNER);
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

    aarch64::set_ttbr0_el1(1, ttbr, true);
    aarch64::set_ttbr1_el1(0, ttbr, true);


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
        //(0b1  << 23) | // EPD1 disable higher half
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
