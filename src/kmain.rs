#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![feature(core_intrinsics)]
#![feature(const_fn)]

#![no_std]
#![no_builtins]

extern crate spin;

#[macro_use]
pub mod print;
pub mod externs;
pub mod panic;

pub mod gpio;
pub mod uart0;
pub mod mbox;
pub mod lfb;

#[no_mangle]
pub extern "C" fn kernel_main(_r0: u32, _r1: u32, _atags: u32) {
    uart0::init();
    println!("Hello Rust Kernel world! 0x{:X}", 0xDEAD);

    let info = lfb::FrameBufferInfo {
        width: 1024,
        height: 768,
        virtual_width: 1024,
        virtual_height: 768,
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

    loop {
        let c = uart0::receive();
        if c != 0 {
            uart0::send(c);
        }
    }
}

