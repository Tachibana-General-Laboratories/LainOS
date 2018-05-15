use core::mem;
use core::alloc::{Alloc, AllocErr, Layout, Opaque};
use core::ptr::NonNull;

use allocator::util::*;
use allocator::linked_list::LinkedList;

const BIN_COUNT: usize = 32;
const END_ALIGN: usize = 8;

/// A simple allocator that allocates based on size classes.
#[derive(Debug)]
pub struct Allocator {
    bin: [LinkedList; BIN_COUNT],

    current: usize,
    end: usize,
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            bin: [LinkedList::new(); BIN_COUNT],
            current: start,
            end,
        }
    }

    fn recycle(&mut self, mut ptr: usize, mut size: usize) {
        for i in (0..BIN_COUNT).rev() {
            let bin = 2usize.pow(3 + i as u32);
            while size % bin != 0 {
                unsafe { self.bin[i].push(ptr as *mut usize); }
                size -= bin;
                ptr -= bin;
            }
        }
    }
}

fn bin_from_layout(layout: Layout) -> usize {
    align_up(layout.size(), layout.align())
        .next_power_of_two()
        .trailing_zeros() as usize - 3
}

unsafe impl<'a> Alloc for Allocator {
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
        let bin = bin_from_layout(layout);
        if bin >= BIN_COUNT {
            return Err(AllocErr);
        }

        let addr = if let Some(addr) = self.bin[bin].pop() {
            align_up(addr as usize, layout.align())
        } else {
            let start = align_up(self.current, layout.align());
            let end = align_up(start + layout.size(), END_ALIGN);

            if end >= self.end {
                ::console::kprintln!("alloc {:x}-{:x} @end: {:x}", start, end, self.end);
                return Err(AllocErr);
            }

            let _ptr = mem::replace(&mut self.current, end);
            //self.recycle(ptr, start - ptr);

            start
        };

        Ok(NonNull::new_unchecked(addr as *mut u8).as_opaque())
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
        let bin = bin_from_layout(layout);
        self.bin[bin].push(ptr.as_ptr() as *mut usize);
    }
}
