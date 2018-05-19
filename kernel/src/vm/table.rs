use core::mem::transmute;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::ops::{Deref, DerefMut};

use sys::volatile::Volatile;
use sys::volatile::prelude::*;

use vm::{Entry, VirtualAddr, PhysicalAddr, Page, Huge};

#[repr(align(4096))]
#[derive(Clone)]
pub struct Table<L: Level> {
    entries: [Entry; 512],
    level: PhantomData<L>,
}

pub trait Level: Sized {
    const ADDR_SHIFT: usize;

    fn physical(&self) -> PhysicalAddr;
    fn to_index(addr: VirtualAddr) -> usize {
        ((addr.as_u64() >> Self::ADDR_SHIFT) & 0x1ff) as usize
    }
}

pub trait Hierarchy: Level {
    type NextLevel: Level + From<Page>;
    #[must_use]
    unsafe fn next_level(e: Entry) -> Self::NextLevel {
        Page::from_entry(e).into()
    }
}

impl<L: Level> Table<L> {
    pub const fn empty() -> Self {
        Self {
            entries: [Entry::INVALID; 512],
            level: PhantomData,
        }
    }

    #[inline]
    pub fn read(&self, addr: VirtualAddr) -> Entry {
        self.get(L::to_index(addr)).read()
    }
    #[inline]
    pub fn write(&mut self, addr: VirtualAddr, entry: Entry) {
        self.get_mut(L::to_index(addr)).write(entry)
    }

    #[inline]
    pub fn get(&self, index: usize) -> &Volatile<Entry> {
        unsafe { transmute(&self.entries[index]) }
    }
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> &mut Volatile<Entry> {
        unsafe { transmute(&mut self.entries[index]) }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=Entry> + 'a {
        self.entries.iter().cloned()
    }
}

impl<L: Hierarchy> Table<L> {
    pub fn next_table<'a>(&'a mut self, addr: VirtualAddr)
        -> Option<&'a mut Table<L::NextLevel>>
    {
        let entry = self.read(addr);
        if entry.is_table() {
            Some(unsafe { &mut *Page::from_entry(entry).into_ptr() })
        } else {
            None
        }
    }

    pub fn next_table_or<'a>(&'a mut self, addr: VirtualAddr, attr: Entry)
        -> Option<&'a mut Table<L::NextLevel>>
    {
        let entry = self.read(addr);
        if entry.is_block() {
            return None;
        }
        let page = if entry.is_table() {
            unsafe { Page::from_entry(entry) }
        } else {
            let page = Page::new_zeroed()?;
            self.write(addr, Entry::table(page.physical()) | Entry::NEED_DROP | attr);
            page
        };
        Some(unsafe { &mut *page.into_ptr() })
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

macro_rules! impl_level {
    ($T:ident[$shift:expr] => ()) => {
        pub struct $T {
            page: Page,
        }

        impl From<Page> for $T {
            fn from(page: Page) -> Self {
                Self { page }
            }
        }

        impl Deref for $T {
            type Target = Table<Self>;
            fn deref(&self) -> &Self::Target { self.page.cast() }
        }

        impl DerefMut for $T {
            fn deref_mut(&mut self) -> &mut Self::Target { self.page.cast_mut() }
        }

        impl Level for $T {
            const ADDR_SHIFT: usize = $shift;

            fn physical(&self) -> PhysicalAddr {
                self.page.physical()
            }
        }
    };

    ($T:ident[$shift:expr] => $next:ty) => {
        impl_level!($T[$shift] => ());

        impl Hierarchy for $T {
            type NextLevel = $next;
        }
    };
}

impl_level!(L0[39] => L1);
impl_level!(L1[30] => L2);
impl_level!(L2[21] => L3);
impl_level!(L3[12] => ());

impl Drop for L0 {
    fn drop(&mut self) {
        panic!("L0 not supported")
    }
}

impl Drop for L1 {
    fn drop(&mut self) {
        for e in self.iter().filter(Entry::is_valid) {
            assert!(e.is_table(), "LARGE block not supported: {}-{} {}", e.is_table(), e.need_drop(), e.addr());
            if e.is_table() {
                unsafe { drop(Self::next_level(e)); }
            }
        }
    }
}

impl Drop for L2 {
    fn drop(&mut self) {
        for e in self.iter().filter(Entry::is_valid) {
            if e.is_table() {
                unsafe { drop(Self::next_level(e)) }
            } else {
                unsafe { drop(Huge::from_entry(e)) }
            }
        }
    }
}

impl Drop for L3 {
    fn drop(&mut self) {
        for e in self.iter().filter(Entry::is_valid) {
            unsafe { drop(Page::from_entry(e)) }
        }
    }
}
