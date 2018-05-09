mod address;
mod entry;
mod page_allocator;


use alloc::Vec;

pub use self::entry::Entry;
pub use self::address::{PhysicalAddr, VirtualAddr};

pub const PAGE_SIZE: usize = 4 * 1024;

pub const BINARY_START: usize = 0;

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
    //pub flags: Map,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl Area {
    pub fn contains(&self, addr: VirtualAddr) -> bool {
        self.start >= addr && addr < self.end
    }

    pub fn intersects(&self, other: Self) -> bool {
        self.contains(other.start) || self.contains(other.end)
    }

    /*
    pub fn is_stack(&self) -> bool {
        self.flags.contains(Map::STACK)
    }
    */
}

pub struct Memory {
    areas: Vec<Area>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            areas: Vec::new()
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            areas: Vec::with_capacity(cap)
        }
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

    /*
    pub fn is_stack_addr(&self, addr: VirtualAddr) -> bool {
        self.find_area(addr).map_or(false, Area::is_stack)
    }
    */
}
