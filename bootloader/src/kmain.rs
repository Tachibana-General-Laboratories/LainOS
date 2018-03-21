#![feature(asm, lang_items)]
#![feature(alloc, allocator_api)]
#![feature(global_allocator)]

extern crate xmodem;
extern crate pi;

extern crate alloc;

pub mod lang_items;

use std::io::{self, Cursor};
use std::slice::from_raw_parts_mut;
use std::fmt::Write;
use xmodem::Xmodem;
use pi::uart::MiniUart;


/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x80000;
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
fn jump_to(addr: *mut u8) -> ! {
    unsafe {
        asm!("br $0" : : "r"(addr as usize));
        loop { asm!("nop" :::: "volatile")  }
    }
}

#[no_mangle]
pub extern "C" fn kmain() {
    let mut uart = MiniUart::new();
    uart.set_read_timeout(750);
    uart.write_str("start bootloader\n");

    let mem: &mut [u8] = unsafe { from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE) };
    let mut mem = Cursor::new(mem);

    uart.write_str("go:\n");
    loop {
        mem.set_position(0);
        match Xmodem::receive(&mut uart, &mut mem) {
            Ok(_) => jump_to(BINARY_START),
            Err(err) => {
                if err.kind() != io::ErrorKind::TimedOut {
                    uart.write_fmt(format_args!("error: {:?}\n", err));
                }
            }
        }
    }
}

#[global_allocator]
static mut ALLOCATOR: Allocator = Allocator;

static mut CURRENT: usize = 0x800_0000;

use alloc::heap::{Alloc, AllocErr, Layout};

#[derive(Debug)]
pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let start = align_up(CURRENT, layout.align());
        unsafe { CURRENT = start + layout.size() };
        Ok(start as *mut u8)
    }
    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        //unimplemented!("dealloc")
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    assert!(align.is_power_of_two() || align == 0);
    let mut down = align_down(addr, align);
    if addr != down {
        down += align
    }
    down
}

pub fn align_down(addr: usize, align: usize) -> usize {
    assert!(align.is_power_of_two() || align == 0);
    addr & !(align - 1)
}
