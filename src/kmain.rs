#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![feature(core_intrinsics)]
#![feature(alloc, allocator_api, global_allocator)]
#![feature(const_fn)]
#![feature(decl_macro)]

//#[macro_use]
//extern crate bitflags;

#[macro_use]
extern crate alloc;
extern crate slab_allocator;
extern crate spin;
extern crate stack_vec;
extern crate volatile;

#[macro_use]
extern crate log;

extern crate sys_fs as fat32;

pub mod console;
pub mod externs;
pub mod panic;
pub mod util;
pub mod exception;
pub mod mmu;
pub mod pi;
pub mod fb;
pub mod shell;

//pub mod sd;
//pub mod sdn;
//pub mod gles;


pub mod fs;


pub mod allocator;

use slab_allocator::LockedHeap;
use alloc::*;

use console::{kprint, kprintln};

#[cfg(not(test))]
#[global_allocator]
static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

const BINARY_START_ADDR: usize = 0x8_0000; // 512kb
const KERNEL_SPACE: usize = 0xFFFFFF80_00000000;

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

pub static FILE_SYSTEM: fs::FileSystem = fs::FileSystem::uninitialized();

#[cfg(not(test))]
pub fn init_heap() {
    let heap_start = KERNEL_SPACE | 0x0F00_0000;
    let heap_end =   KERNEL_SPACE | 0x1000_0000;
    unsafe {
        ALLOCATOR.init(heap_start, heap_end - heap_start);
    }
}

#[no_mangle]
#[inline(never)]
#[cfg(not(test))]
pub extern "C" fn kernel_main() -> ! {
    kprintln!("init console");

    unsafe {
        let f: u32 = (1 << 0) | (1 << 1) | (1 << 2);
        asm!("msr cntv_ctl_el0, $0" : : "r"(f) : : "volatile");

        let level: u32;
        // read the current level from system register
        asm!("mrs $0, CurrentEL" : "=r" (level) : : : "volatile");
        kprintln!("Current EL is: 0x{:X} [0x{:X}]", (level >> 2) & 3, level);
    }

    kprintln!("init mmu");
    unsafe { mmu::init(); }

    kprintln!("init heap");
    init_heap();

    kprintln!("init fs");
    FILE_SYSTEM.initialize();

    test_timers();

    kprintln!("init fb");
    init_fb();

    //kprintln!("init gles: {:?}", gles::InitV3D());

    shell::shell("> ")
}

fn init_fb() {
    if let Some(mut fb) = fb::FrameBuffer::new(480, 320, 32) {
        fb.fill_rgba(0x000000);
        fb::font().uprint(&mut fb, 13, 5, "Prepare uranus!", 0x00FF00, 0x0000FF);
        fb::font().uprint(&mut fb, 13, 6, "Prepare uranus!", 0xFF0000, 0x0000FF);
        fb::font().uprint(&mut fb, 11, 8, "< Prepare uranus! >", 0xFF0000, 0x000000);

        fb::font().uprint(&mut fb, 1, 0, "  .  ",  0xFFFFFF, 0x000000);
        fb::font().uprint(&mut fb, 1, 1, "< 0 >",  0xFFFFFF, 0x000000);
        fb::font().uprint(&mut fb, 1, 2, "./ \\.", 0xFFFFFF, 0x000000);
    } else {
        kprintln!("Unable to set screen resolution");
    }
}

fn test_timers() {
    kprint!("Waiting 1000000 CPU cycles (ARM CPU): ");
    util::wait_cycles(1000000);
    kprintln!("OK");

    kprint!("Waiting 1000000 microsec (ARM CPU): ");
    util::wait_msec(1000000);
    kprintln!("OK");

    kprint!("Waiting 1000000 microsec (BCM System Timer): ");
    if pi::timer::current_time() == 0 {
        kprintln!("Not available");
    } else {
        pi::timer::spin_sleep_us(1000000);
        kprintln!("OK");
    }
}
