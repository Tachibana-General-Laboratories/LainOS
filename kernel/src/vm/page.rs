use core::ptr::NonNull;
use core::mem::forget;
use alloc::allocator::{Alloc, Layout, Global};

use vm::{Entry, PhysicalAddr, p2v, v2p};

pub const PAGESZ: usize = 4096;

pub struct Page {
    pub(in vm) ptr: NonNull<[u8; PAGESZ]>,
    need_drop: bool,
}

impl Page {
    fn layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(PAGESZ, PAGESZ) }
    }

    pub fn new() -> Option<Self> {
        unsafe {
            let ptr = Global.alloc(Self::layout()).ok()?.cast();
            Some(Self { ptr, need_drop: true })
        }
    }

    pub fn new_zeroed() -> Option<Self> {
        unsafe {
            let ptr = Global.alloc_zeroed(Self::layout()).ok()?.cast();
            Some(Self { ptr, need_drop: true })
        }
    }

    pub fn physical(&self) -> PhysicalAddr {
        v2p(self.ptr.into()).expect("pages MUST be in kernel space")
    }

    #[must_use]
    pub unsafe fn from_entry(e: Entry) -> Self {
        let ptr = p2v(e.addr()).as_mut_ptr();
        let ptr = NonNull::new_unchecked(ptr);
        Self { ptr, need_drop: e.need_drop() }
    }

    pub fn into_ptr<T>(self) -> *mut T {
        let ptr = self.ptr.cast().as_ptr();
        forget(self);
        ptr
    }

    pub fn cast<T>(&self) -> &T {
        unsafe { &*self.ptr.cast().as_ptr() }
    }

    pub fn cast_mut<T>(&mut self) -> &mut T {
        unsafe { &mut *self.ptr.cast().as_ptr() }
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        if self.need_drop {
            unsafe { Global.dealloc(self.ptr.as_opaque(), Self::layout()) }
        }
    }
}

impl Into<PhysicalAddr> for Page {
    fn into(self) -> PhysicalAddr {
        let ptr = self.physical();
        forget(self);
        ptr
    }
}
