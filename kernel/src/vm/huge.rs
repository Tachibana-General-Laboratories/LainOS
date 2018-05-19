use core::ptr::NonNull;
use alloc::allocator::{Alloc, Layout, Global};

use vm::{Entry, p2v};

pub const HUGESZ: usize = 4096 * 512;
type Inner = [u8; HUGESZ];

pub struct Huge {
    pub(in vm) ptr: NonNull<Inner>,
    need_drop: bool,
}

impl Huge {
    fn layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(HUGESZ, HUGESZ) }
    }

    pub fn new() -> Option<Self> {
        unsafe {
            let ptr = Global.alloc(Self::layout()).ok()?.cast();
            Some(Self { ptr, need_drop: true })
        }
    }

    #[must_use]
    pub unsafe fn from_entry(e: Entry) -> Self {
        let ptr = p2v(e.addr()).as_mut_ptr();
        let ptr = NonNull::new_unchecked(ptr);
        Self { ptr, need_drop: e.need_drop() }
    }
}

impl Drop for Huge {
    fn drop(&mut self) {
        if self.need_drop {
            unsafe { Global.dealloc(self.ptr.as_opaque(), Self::layout()) }
        }
    }
}
