#![feature(lang_items)]
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
#[cfg(test)] extern crate core;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate alloc;
//extern crate spin;

#[cfg(not(test))] extern crate sys;
#[cfg(not(test))] extern crate pi;
#[cfg(not(test))] extern crate vfat;

#[cfg(not(test))] pub mod console;
#[cfg(not(test))] pub mod elf;

#[cfg(not(test))] pub mod aarch64;
#[cfg(not(test))] pub mod process;
#[cfg(not(test))] pub mod traps;
#[cfg(not(test))] pub mod vm;
#[cfg(not(test))] pub mod panic;
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

//const BINARY_START_ADDR: usize = 0x8_0000; // 512kb

/*
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
*/


#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kernel_main() -> ! {
    //dbg::hello();
    //dbg::print_atags();
    //dbg::print_memory();

    vm::enable_mmu();

    unsafe {
        asm!("
            bl      0f
            b       1f
        0:
            mov     x1, sp
            movk    x1, #0xFFFF, LSL #48
            movk    x1, #0xFF80, LSL #32
            mov     sp, x1

            mrs     x1, VBAR_EL1
            movk    x1, #0xFFFF, LSL #48
            movk    x1, #0xFF80, LSL #32
            msr     VBAR_EL1, x1

            movk    x30, #0xFFFF, LSL #48
            movk    x30, #0xFF80, LSL #32
            ret
        1:
        " : : : "x1" : "volatile");
    }

    ALLOCATOR.initialize();
    //dbg::test_alloc();
    vm::initialize();
    FILE_SYSTEM.initialize();
    //dbg::test_fs();
    //dbg::test_timers();

    //init_fb(480, 320);
    //init_fb(1920, 1080);
    //shell::shell("kernel> ");

    kprintln!("");
    kprintln!("---- EL0: ----");

    pi::timer::spin_sleep_ms(500);

    use pi::interrupt::{Controller, Interrupt};
    use pi::timer::tick_in;

    Controller::new().enable(Interrupt::Timer1);
    tick_in(process::TICK);

    SCHEDULER.start()
}


#[cfg(not(test))]
mod dbg {
    use console::*;

    pub fn hello() {
        use pi::gpio::Gpio;
        use pi::timer::spin_sleep_ms;
        let mut pin = Gpio::new(16).into_output();
        pin.set();
        spin_sleep_ms(500);
        pin.clear();
        spin_sleep_ms(500);
        pin.set();

        let el = unsafe { ::aarch64::current_el() };
        let cpuid = unsafe { ::aarch64::affinity() };
        kprintln!("-------------------------------");
        kprintln!("Start KERNEL main at [CPU{} EL{}]", cpuid, el);
        kprintln!("-------------------------------");
        kprintln!();
    }

    pub fn print_atags() {
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

    pub fn print_memory() {
        use aarch64;
        kprintln!("{:?}", aarch64::AArch64::new());
        kprintln!("MAIR_EL1: {:?}", aarch64::MemoryAttributeIndirectionRegister::el1());
    }

    pub fn test_alloc() {
        kprint!("allocator test: ");
        use alloc::string::{String, ToString};
        let mut fuck: String = "##fuck".to_string();
        for _ in 0..12 {
            fuck += " fuck";
        }
        fuck += "##";
        kprintln!("{:p}: `{}`", fuck.as_ptr(), fuck);
    }

    pub fn test_fs() {
        kprint!("test fs: ls /");
        use vfat::traits::*;

        match ::FILE_SYSTEM.open_dir("").and_then(|e| e.entries()) {
            Ok(entries) => {
                for e in entries {
                    kprintln!("   /{}", e.name());
                }
            }
            Err(err) => kprintln!("ls: {:?}", err),
        }
    }

    pub fn test_timers() {
        use pi;

        kprintln!("test timers:");

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

    pub fn test_fb(w: u32, h: u32) {
        use fb;
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
}
