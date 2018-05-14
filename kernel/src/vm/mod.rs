mod address;
mod entry;
mod table;

use alloc::vec::Vec;
use alloc::boxed::Box;

use sys::volatile::prelude::*;
use sys::volatile::Volatile;

use allocator::util::*;

pub use self::entry::Entry;
pub use self::table::{Table, Level, Level0, Level1, Level2, Level3};
pub use self::address::{PhysicalAddr, VirtualAddr};

pub const PAGESZ: usize = 4096;
pub const HUGESZ: usize = 4096 * 512;

use ALLOCATOR;
use alloc::allocator::{Alloc, Layout};

//use super::{Level, Table, PAGESZ, HUGESZ};

pub unsafe fn alloc_table<'a, L: Level>() -> Option<&'a mut Table<L>> {
    let layout = Layout::from_size_align_unchecked(PAGESZ, PAGESZ);
    let mem = (&ALLOCATOR).alloc_zeroed(layout).ok()?;
    let table: &mut Table<_> = &mut *(mem.cast().as_ptr());
    Some(table)
}

pub unsafe fn alloc_page() -> Option<PhysicalAddr> {
    let layout = Layout::from_size_align_unchecked(PAGESZ, PAGESZ);
    let mem = (&ALLOCATOR).alloc(layout).ok()?;
    Some(mem.cast::<usize>().as_ptr().into())
}
pub unsafe fn alloc_huge() -> Option<PhysicalAddr> {
    let layout = Layout::from_size_align_unchecked(HUGESZ, HUGESZ);
    let mem = (&ALLOCATOR).alloc(layout).ok()?;
    Some(mem.cast::<usize>().as_ptr().into())
}

pub struct Memory<'a> {
    pub root: &'a mut Table<Level1>,
    pub areas: Vec<Area>,
}

impl<'a> Memory<'a> {
    pub fn new() -> Option<Self> {
        Some(Self {
            root: unsafe { alloc_table()? },
            areas: Vec::new()
        })
    }

    pub fn ttbr(&mut self) -> PhysicalAddr {
        ((&mut *self.root) as *mut Table<Level1>).into()
    }

    pub fn area(self, start: usize, end: usize) -> AreaBuilder<'a> {
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
        let (a1, a2, _) = self.find_area(v).unwrap().entries();
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


//pub const BINARY_START: usize = 0;

pub struct AreaBuilder<'mem> {
    mem: Memory<'mem>,
    area: Area,
}

impl<'mem> AreaBuilder<'mem> {
    fn new(mem: Memory<'mem>, start: usize, end: usize) -> Self {
        let start = VirtualAddr::from((start as *mut u8));
        let end = VirtualAddr::from((end as *mut u8));
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

    pub fn create(self) -> Memory<'mem> {
        let Self { mut mem, area } = self;
        ::console::kprintln!("area create: {}-{}", area.start, area.end);
        mem.areas.push(area);
        mem
    }

    pub fn alloc(self) -> Memory<'mem> {
        let Self { mut mem, area } = self;
        ::console::kprintln!("area alloc: {}-{}", area.start, area.end);
        let mut addr = area.start.as_usize();
        let end = area.end.as_usize();
        mem.areas.push(area);
        while addr < end {
            let v: VirtualAddr = (addr as *mut u8).into();
            if addr % HUGESZ == 0 && (end - addr) >= HUGESZ {
                unsafe { mem.alloc_huge(v).unwrap() };
                addr += HUGESZ;
                continue;
            }
            unsafe { mem.alloc_page(v).unwrap() };
            addr += PAGESZ;
        }
        mem
    }

    pub fn map(self) -> Memory<'mem> {
        let Self { mut mem, area } = self;
        ::console::kprintln!("area map: {}-{}", area.start, area.end);
        let mut addr = area.start.as_usize();
        let end = area.end.as_usize();
        mem.areas.push(area);
        while addr < end {
            let v: VirtualAddr = (addr as *mut u8).into();
            let p: PhysicalAddr = (addr as *mut u8).into();
            if addr % HUGESZ == 0 && (end - addr) >= HUGESZ {
                unsafe { mem.map_huge(v, p); }
                addr += HUGESZ;
                continue;
            }
            unsafe { mem.map_page(v, p); }
            addr += PAGESZ;
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

/*
pub struct Memory {
}

impl Memory {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            areas: Vec::with_capacity(cap)
        }
    }

    /*
    pub fn is_stack_addr(&self, addr: VirtualAddr) -> bool {
        self.find_area(addr).map_or(false, Area::is_stack)
    }
    */
}
*/
