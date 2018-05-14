use core::mem;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use sys::volatile::Volatile;
use sys::volatile::prelude::*;

use vm::{Entry, VirtualAddr};

#[repr(align(4096))]
#[derive(Clone)]
pub struct Table<L: Level> {
    entries: [Entry; 512],
    level: PhantomData<L>,
}

pub trait Level: Sized {
    const ADDR_SHIFT: usize;
    type NextLevel;

    fn to_index(addr: VirtualAddr) -> usize {
        ((addr.as_u64() >> Self::ADDR_SHIFT) & 0x1ff) as usize
    }
}

pub enum Level0 {}
pub enum Level1 {}
pub enum Level2 {}
pub enum Level3 {}

impl Level for Level0 {
    const ADDR_SHIFT: usize = 39;
    type NextLevel = Level1;
}
impl Level for Level1 {
    const ADDR_SHIFT: usize = 30;
    type NextLevel = Level2;
}
impl Level for Level2 {
    const ADDR_SHIFT: usize = 21;
    type NextLevel = Level3;
}
impl Level for Level3 {
    const ADDR_SHIFT: usize = 12;
    type NextLevel = ();
}

impl<L: Level> Table<L> {
    pub const fn empty() -> Self {
        Self {
            entries: [Entry::INVALID; 512],
            level: PhantomData,
        }
    }

    pub fn get(&self, index: usize) -> &Volatile<Entry> {
        unsafe { mem::transmute(&self.entries[index]) }
    }
    pub fn get_mut(&mut self, index: usize) -> &mut Volatile<Entry> {
        unsafe { mem::transmute(&mut self.entries[index]) }
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=Entry> + 'a {
        self.entries.iter().cloned()
    }
}

impl<L: Level> Table<L>
    where L::NextLevel: Level
{
    /*
    pub unsafe fn next_table(&mut self, addr: VirtualAddr) -> Option<&mut Table<L::NextLevel>> {
        self[addr].read().as_table::<L::NextLevel>()
    }
    */

    pub unsafe fn next_table_or<'a, F>(&'a mut self, addr: VirtualAddr, f: F)
        -> Option<&'a mut Table<L::NextLevel>>
        where F: FnOnce() -> Option<(Entry, &'a mut Table<L::NextLevel>)>
    {
        self[addr].read().as_table::<L::NextLevel>()
            .or_else(|| {
                let (attr, next) = f()?;
                self[addr].write(Entry::table((next as *mut _).into()) | attr);
                Some(next)
            })
    }
}

impl<L: Level> Index<VirtualAddr> for Table<L> {
    type Output = Volatile<Entry>;
    fn index(&self, addr: VirtualAddr) -> &Self::Output {
        self.get(L::to_index(addr))
    }
}

impl<L: Level> IndexMut<VirtualAddr> for Table<L> {
    fn index_mut(&mut self, addr: VirtualAddr) -> &mut Self::Output {
        self.get_mut(L::to_index(addr))
    }
}

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
