use core::fmt;
use core::ptr::NonNull;

use alloc::allocator::{Global, Alloc, Layout};
use vm::{PhysicalAddr, v2p};

/// A process stack. The default size is 1M1B with an alignment of 16 bytes.
pub struct Stack {
    ptr: NonNull<[u8; Stack::SIZE]>
}

unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}

impl Stack {
    /// The default stack size is 1MiB.
    pub const SIZE: usize = 1 << 20;

    /// The default stack alignment is 16 bytes.
    pub const ALIGN: usize = 16;

    /// The default layout for a stack.
    fn layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(Self::SIZE, Self::ALIGN) }
    }

    /// Returns a newly allocated process stack, zeroed out, if one could be
    /// successfully allocated. If there is no memory, or memory allocation
    /// fails for some other reason, returns `None`.
    pub fn new() -> Option<Self> {
        unsafe {
            let ptr = Global.alloc_zeroed(Self::layout()).ok()?.cast();
            Some(Stack { ptr })
        }
    }

    /// Internal method to cast to a `*mut u8`.
    unsafe fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr.cast().as_ptr()
    }

    /// Returns the physical address of top of the stack.
    pub fn top(&self) -> PhysicalAddr {
        unsafe { v2p(self.as_mut_ptr().add(Self::SIZE).into()).expect("Stack MUST be in a kernel space") }
    }

    /// Returns the physical address of bottom of the stack.
    pub fn bottom(&self) -> PhysicalAddr {
        unsafe { v2p(self.as_mut_ptr().into()).expect("Stack MUST be in a kernel space") }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        unsafe {
            Global.dealloc(self.ptr.as_opaque(), Self::layout())
        }
    }
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Stack")
            .field("top", &self.top())
            .field("bottom", &self.bottom())
            .field("size", &Self::SIZE)
            .finish()
    }
}
