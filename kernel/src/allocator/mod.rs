pub mod linked_list;
pub mod util;

#[path = "bin.rs"]
mod imp;

#[cfg(test)]
mod tests;

use sys::Mutex;
use pi::atags::Atags;
use core::alloc::{Alloc, GlobalAlloc, AllocErr, Layout, Opaque};
use core::ptr::NonNull;

/// Thread-safe (locking) wrapper around a particular memory allocator.
#[derive(Debug)]
pub struct Allocator(Mutex<Option<imp::Allocator>>);

impl Allocator {
    /// Returns an uninitialized `Allocator`.
    ///
    /// The allocator must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        Allocator(Mutex::new(None))
    }

    /// Initializes the memory allocator.
    ///
    /// # Panics
    ///
    /// Panics if the system's memory map could not be retrieved.
    pub fn initialize(&self) {
        let (start, end) = memory_map().expect("failed to find memory map");
        let size = end - start;
        let heap = imp::Allocator::new(start, size);
        *self.0.lock().unwrap() = Some(heap);
    }
}

unsafe impl<'a> Alloc for &'a Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<Opaque>, AllocErr> {
        self.0.lock().unwrap().as_mut().expect("allocator uninitialized").alloc(layout)
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: NonNull<Opaque>, layout: Layout) {
        self.0.lock().unwrap().as_mut().expect("allocator uninitialized").dealloc(ptr, layout);
    }
}

unsafe impl<'a> GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        let p = self.0.lock().unwrap().as_mut().expect("allocator uninitialized").alloc(layout);
        if let Ok(p) = p {
            p.as_ptr()
        } else {
            ::core::ptr::null_mut() as *mut u8 as *mut Opaque
        }
    }
    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        self.0.lock().unwrap().as_mut().expect("allocator uninitialized").dealloc(ptr, layout);
    }
}

extern "C" {
    static _end: u8;
}

/// Returns the (start address, end address) of the available memory on this
/// system if it can be determined. If it cannot, `None` is returned.
///
/// This function is expected to return `Some` under all normal cirumstances.
fn memory_map() -> Option<(usize, usize)> {
    const MIN_HEAP_SIZE: usize = 4096;
    let binary_end = unsafe { (&_end as *const u8) as u32 };
    let start = util::align_up(binary_end as usize, MIN_HEAP_SIZE);

    let end = Atags::get()
        .filter_map(|t| t.mem())
        .map(|t| (t.start + t.size) as usize)
        .max()
        .unwrap_or(0x4000_0000);
    Some((start, end))
}
