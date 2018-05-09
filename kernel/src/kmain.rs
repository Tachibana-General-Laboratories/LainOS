#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(never_type)]
#![feature(unique)]
#![feature(pointer_methods)]
#![feature(naked_functions)]
#![feature(fn_must_use)]
#![feature(alloc_system, alloc, allocator_api, global_allocator)]
#![feature(ptr_internals)]
#![feature(nonzero)]


#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate bitflags;

//extern crate core;
#[macro_use]
extern crate alloc;
//extern crate slab_allocator;
//extern crate spin;

#[cfg(not(test))] extern crate sys;
#[cfg(not(test))] extern crate pi;
#[cfg(not(test))] extern crate sys_fs as fat32;


/*
#[macro_use]
extern crate log;
*/

pub mod console;
pub mod elf;

#[cfg(not(test))] pub mod aarch64;
#[cfg(not(test))] pub mod process;
#[cfg(not(test))] pub mod traps;
#[cfg(not(test))] pub mod vm;
#[cfg(not(test))] pub mod panic;
#[cfg(not(test))] pub mod mmu;
#[cfg(not(test))] pub mod fb;
#[cfg(not(test))] pub mod user;

//pub mod sd;
//pub mod sdn;
//pub mod gles;
#[cfg(not(test))] pub mod fs;
pub mod allocator;

#[cfg(not(test))] use console::{kprint, kprintln};

#[cfg(not(test))] use allocator::Allocator;
#[cfg(not(test))] use fs::FileSystem;
#[cfg(not(test))] use process::GlobalScheduler;

#[global_allocator]
#[cfg(not(test))] pub static ALLOCATOR: Allocator = Allocator::uninitialized();
#[cfg(not(test))] pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();
#[cfg(not(test))] pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();

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


#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kernel_main() -> ! {
    // hello blink
    {
        use pi::gpio::Gpio;
        use pi::timer::spin_sleep_ms;
        let mut pin = Gpio::new(16).into_output();
        pin.set();
        spin_sleep_ms(500);
        pin.clear();
        spin_sleep_ms(500);
        pin.set();
    }

    let el = unsafe { aarch64::current_el() };
    let cpuid = unsafe { aarch64::affinity() };
    kprintln!("-------------------------------");
    kprintln!("Start KERNEL main at [CPU{} EL{}]", cpuid, el);
    kprintln!("-------------------------------");
    kprintln!();


    print_atags();

    kprintln!("{:?}", aarch64::AArch64::new());
    kprintln!("MAIR_EL1: {:?}", aarch64::MemoryAttributeIndirectionRegister::el1());

    /*
    init_logger().unwrap();
    info!("test logger");
    */

    kprintln!("initialize mmu");
    unsafe { mmu::init_mmu(); }


    {
        kprint!("allocator init: ");
        ALLOCATOR.initialize();
        kprintln!("OK");

        kprint!("file system init: ");
        FILE_SYSTEM.initialize();
        kprintln!("OK");
    }


    //test_timers();

    //init_fb(480, 320);
    //init_fb(1920, 1080);
    //

    //shell::shell("kernel> ");

    {
        kprintln!("---- EL0: ----");

        use pi::interrupt::{Controller, Interrupt};
        use pi::timer::tick_in;

        Controller::new().enable(Interrupt::Timer1);
        tick_in(process::TICK);

        SCHEDULER.start()
    }
}


#[cfg(not(test))]
fn print_atags() {
    use pi::atags::*;
    for atag in Atags::get() {
        match atag {
            Atag::Mem(Mem { start, size }) => {
                kprintln!("Atag::Mem {:#016X}-{:#016X} [size: {:#X}]", start, start + size, size);
            }
            Atag::Cmd(s) => {
                kprintln!("Atag::Cmd:");
                for s in s.split_terminator(' ') {
                    kprintln!("  {}", s);
                }
            }
            _ => kprintln!("Atag: {:?}", atag),
        }
    }
}


#[cfg(not(test))]
fn init_fb(w: u32, h: u32) {
    kprintln!("initialize fb {}x{}", w, h);
    if let Some(mut fb) = fb::FrameBuffer::new(w, h, 32) {
        fb.fill_rgba(0x330000);

        fb::font().uprint(&mut fb, 13, 5, "Prepare uranus!", 0x00FF00, 0x0000FF);
        fb::font().uprint(&mut fb, 13, 6, "Prepare uranus!", 0xFF0000, 0x0000FF);
        fb::font().uprint(&mut fb, 11, 8, "< Prepare uranus! >", 0xFF0000, 0x000000);

        fb::font().uprint(&mut fb, 1, 0, "  .  ",  0xFFFFFF, 0x000000);
        fb::font().uprint(&mut fb, 1, 1, "< 0 >",  0xFFFFFF, 0x000000);
        fb::font().uprint(&mut fb, 1, 2, "./ \\.", 0xFFFFFF, 0x000000);

        /*
        kprint!("init gles:");
        if let Ok(_) = gles::InitV3D() {
            kprintln!("OK");
            gles::test_triangle(1920, 1080, pi::mbox::arm2gpu(fb.addr()) as u32);
        } else {
            kprintln!("ERR");
        }
        */

    } else {
        kprintln!("Unable to set screen resolution");
    }
}

#[cfg(not(test))]
fn test_timers() {
    kprint!("Waiting 1000000 CPU cycles (ARM CPU): ");
    pi::common::spin_sleep_cycles(1000000);
    kprintln!("OK");

    kprint!("Waiting 1000000 microsec (ARM CPU): ");
    pi::common::spin_sleep_us(1000000);
    kprintln!("OK");

    kprint!("Waiting 1000000 microsec (BCM System Timer): ");
    if pi::timer::current_time() == 0 {
        kprintln!("Not available");
    } else {
        pi::timer::spin_sleep_us(1000000);
        kprintln!("OK");
    }
}

/*

use log::{Record, Level, Metadata};
use log::{SetLoggerError, LevelFilter};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        //metadata.level() <= Level::Info
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        kprintln!("{: >20}:{: <4} {:>5} -- {}",
            record.file().unwrap_or(""),
            record.line().unwrap_or(0),
            record.level(),
            record.args(),
        );
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&LOGGER)
}
*/
