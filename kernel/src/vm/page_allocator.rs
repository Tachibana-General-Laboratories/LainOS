use allocator::util::*;
use allocator::linked_list::LinkedList;

use vm::{PhysicalAddr, PAGE_SIZE};

struct PageAllocator {
    pages: LinkedList,
    current: usize,
    end: usize,
}

impl PageAllocator {
    pub fn alloc(&mut self) -> Option<PhysicalAddr> {
        self.pages.pop().or_else(|| {
                if self.current == self.end {
                    return None;
                }
                let ptr = self.current;
                self.current += PAGE_SIZE;
                Some(ptr as *mut usize)
            })
            .map(PhysicalAddr::from)
    }
    pub fn dealloc(&mut self, ptr: PhysicalAddr) {
        unsafe { self.pages.push(ptr.as_ptr() as *mut usize); }
    }
}
