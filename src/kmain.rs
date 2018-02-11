#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![feature(core_intrinsics)]
#![feature(const_fn)]

#![no_std]
#![no_builtins]

extern crate spin;
extern crate stack_vec;

#[macro_use]
pub mod print;
pub mod externs;
pub mod panic;

pub mod gpio;

pub mod delays;
pub mod uart0;
pub mod mbox;
pub mod lfb;
pub mod power;
pub mod shell;
//pub mod sd;

#[no_mangle]
#[inline(never)]
pub extern "C" fn kernel_main() -> ! {
    uart0::init();

    println!("      .  ");
    println!("    < 0 >");
    println!("    ./ \\.");
    println!("");

    println!("Hello Rust Kernel world! 0x{:X}", 0xDEAD);

    unsafe {
        let level: u32;
        // read the current level from system register
        asm!("mrs $0, CurrentEL" : "=r" (level) : : : "volatile");
        println!("Current EL is: 0x{:X} [0x{:X}]", (level >> 2) & 3, level);
    }

    {
        print!("Waiting 1000000 CPU cycles (ARM CPU): ");
        delays::wait_cycles(1000000);
        println!("OK");

        print!("Waiting 1000000 microsec (ARM CPU): ");
        delays::wait_msec(1000000);
        println!("OK");

        print!("Waiting 1000000 microsec (BCM System Timer): ");
        if delays::get_system_timer() == 0 {
            println!("Not available\n");
        } else {
            delays::wait_msec_st(1000000);
            println!("OK\n");
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
