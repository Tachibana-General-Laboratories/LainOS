#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(alloc, allocator_api)]
#![feature(global_allocator)]
#![feature(decl_macro)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate alloc;
extern crate slab_allocator;
extern crate spin;
extern crate stack_vec;
extern crate volatile;

#[macro_use]
#[cfg(not(test))]
pub mod print;

//#[cfg(not(test))]
//pub mod console;
pub mod externs;
#[cfg(not(test))]
pub mod panic;

#[cfg(not(test))]
pub mod util;

#[cfg(not(test))]
pub mod exception;
#[cfg(not(test))]
pub mod mmu;

#[cfg(not(test))]
pub mod pi;
#[cfg(not(test))]
pub mod fb;
#[cfg(not(test))]
pub mod shell;
//pub mod sd;
pub mod sdn;


pub mod allocator;

use slab_allocator::LockedHeap;
use alloc::*;

#[cfg(not(test))]
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

#[cfg(not(test))]
pub fn init_heap() {
    // TODO why?
    let heap_start = 0x0F00_0000;
    let heap_end =   0x1000_0000;
    let heap_size = heap_end - heap_start;
    unsafe {
        ALLOCATOR.init(heap_start, heap_size);
    }
}

#[no_mangle]
#[inline(never)]
#[cfg(not(test))]
pub extern "C" fn kernel_main() -> ! {
    init_heap();

    pi::uart0::Uart0::new().initialize();

    println!("      .  ");
    println!("    < 0 >");
    println!("    ./ \\.");
    println!("");

    let mut s = String::from("fucking string!");

    println!("Hello Rust Kernel world! 0x{:X} {}", 0xDEAD, s);

    unsafe {
        // initialize EMMC and detect SD card type
        if sdn::sd_init() == 0  {
            // read the master boot record after our bss segment
            let p = 0x00F0_0000 as *mut u8;
            let len = sdn::sd_readblock(0, p, 1) as usize;
            if len != 0 {
                // dump it to serial console
                util::dump(p, len);
            }

            let p = p.offset(len as isize);
            let len = sdn::sd_readblock(1, p, 1) as usize;
            if len != 0 {
                // dump it to serial console
                util::dump(p, len);
            }
        }
    }


    if true {
        unsafe {
            mmu::init();
        }

            /*
            // generate a Data Abort with a bad address access
            let mut r = gpio::Mmio::new(0xFFFF_FFFF_FF00_0000);
            r.write(1u32);
            */

        const KERNEL_UART0: usize = 0xFFFFFF80_3F201000;

        let s = format!("fuck {:016X}\n", KERNEL_UART0);
        let mut uart = unsafe { pi::uart0::Uart0::from_addr(KERNEL_UART0) };
        for c in s.chars() {
            uart.send(c as u8);
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

    let info = fb::FrameBufferInfo {
        width: 360,
        height: 640,
        virtual_width: 360,
        virtual_height: 640,
        x_offset: 0,
        y_offset: 0,
        depth: 32,
        rgb: false,
    };

    match fb::init(info) {
        Some(fb) =>  {
            fb.fill_rgba(0xFF0000);
            fb::font().uprint(&fb, 15, 5, "Prepare uranus!", 0x00FF00, 0x0000FF);
            fb::font().uprint(&fb, 15, 6, "Prepare uranus!", 0xFF0000, 0x0000FF);
            fb::font().uprint(&fb, 13, 8, "< Prepare uranus! >", 0xFF0000, 0x000000);

            fb::font().uprint(&fb, 20, 10, "  .  ",  0x000000, 0xFF0000);
            fb::font().uprint(&fb, 20, 11, "< 0 >",  0x000000, 0xFF0000);
            fb::font().uprint(&fb, 20, 12, "./ \\.", 0x000000, 0xFF0000);

        }
        None => println!("Unable to set screen resolution to 1024x768x32"),
    }

    shell::shell("> ")
}
