#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(alloc, allocator_api)]
#![feature(global_allocator)]
#![feature(decl_macro)]

#![no_std]
#![no_builtins]

#[macro_use]
extern crate alloc;
extern crate slab_allocator;
extern crate spin;
extern crate stack_vec;
extern crate volatile;

#[macro_use]
pub mod print;
pub mod externs;
pub mod panic;

pub mod util;

pub mod exception;
pub mod mmu;

pub mod gpio;
pub mod pi;
pub mod uart0;
pub mod mbox;
pub mod lfb;
pub mod power;
pub mod shell;
//pub mod sd;

use slab_allocator::LockedHeap;
use alloc::*;

#[global_allocator]
static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

const BINARY_START_ADDR: usize = 0x8_0000; // 512kb

const ADDR_1MB: usize   = 0x0010_0000;
const ADDR_2MB: usize   = 0x0020_0000;
const ADDR_4MB: usize   = 0x0040_0000;
const ADDR_8MB: usize   = 0x0080_0000;
const ADDR_16MB: usize  = 0x0100_0000;
const ADDR_32MB: usize  = 0x0200_0000;
const ADDR_64MB: usize  = 0x0400_0000;
const ADDR_128MB: usize = 0x0800_0000;
const ADDR_256MB: usize = 0x1000_0000;
const ADDR_512MB: usize = 0x2000_0000;
const ADDR_1GB: usize   = 0x4000_0000;
const ADDR_2GB: usize   = 0x8000_0000;

pub fn init_heap() {
    // TODO why?
    let heap_start = ADDR_128MB;
    let heap_end = ADDR_256MB;
    let heap_size = heap_end - heap_start;
    unsafe {
        ALLOCATOR.init(heap_start, heap_size);
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn kernel_main() -> ! {
    init_heap();

    uart0::init();

    println!("      .  ");
    println!("    < 0 >");
    println!("    ./ \\.");
    println!("");

    let s = String::from("fucking string!");

    println!("Hello Rust Kernel world! 0x{:X} {}", 0xDEAD, s);

    util::dump(s.as_str().as_ptr(), s.len());

    if false {
        unsafe {
            mmu::init();

            // generate a Data Abort with a bad address access
            let mut r = gpio::Mmio::new(0xFFFF_FFFF_FF00_0000);
            r.write(1u32);
            println!("bad {}", r.read());
        }
    }

    unsafe {
        let level: u32;
        // read the current level from system register
        asm!("mrs $0, CurrentEL" : "=r" (level) : : : "volatile");
        println!("Current EL is: 0x{:X} [0x{:X}]", (level >> 2) & 3, level);
    }

    {
        print!("Waiting 1000000 CPU cycles (ARM CPU): ");
        util::wait_cycles(1000000);
        println!("OK");

        print!("Waiting 1000000 microsec (ARM CPU): ");
        util::wait_msec(1000000);
        println!("OK");

        print!("Waiting 1000000 microsec (BCM System Timer): ");
        if pi::timer::current_time() == 0 {
            println!("Not available");
        } else {
            pi::timer::spin_sleep_us(1000000);
            println!("OK");
        }
    }

    let info = lfb::FrameBufferInfo {
        width: 360,
        height: 640,
        virtual_width: 360,
        virtual_height: 640,
        x_offset: 0,
        y_offset: 0,
        depth: 32,
        rgb: false,
    };

    if let Some(lfb) = lfb::init(info) {
        lfb.fill_rgba(0xFF0000);
        unsafe {
            use core::mem::transmute;
            let font = transmute::<*const u8, &'static lfb::Font>(lfb::FONT.as_ptr());
            font.uprint(lfb, 10, 5, "Prepare uranus!", 0x00FF00, 0x0000FF);
        }
    } else {
        println!("Unable to set screen resolution to 1024x768x32");
    }

    println!("you have no choice");

    shell::shell("> ");
}
