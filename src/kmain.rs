#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![feature(core_intrinsics)]
#![feature(const_fn)]

#![no_std]
#![no_builtins]

use core::intrinsics::{volatile_load, volatile_store};
use core::slice::from_raw_parts_mut;

pub mod externs;
pub mod panic;

pub mod gpio;
pub mod uart0;
pub mod mbox;
pub mod lfb;

#[no_mangle]
pub extern "C" fn kernel_main(_r0: u32, _r1: u32, _atags: u32) {
    uart0::init();
    uart0::puts("Hello Rust Kernel world!\n");

    let info = lfb::FrameBufferInfo {
        width: 1024,
        height: 768,
        virtual_width: 1024,
        virtual_height: 768,
        x_offset: 0,
        y_offset: 0,
        depth: 32,
        rgb: true,
    };

    if let Some(lfb) = lfb::init(info) {
        lfb.fill_rgba(0xCC6666_99);
    } else {
        uart0::puts("Unable to set screen resolution to 1024x768x32\n");
    }

    loop {
        let c = uart0::receive();
        if c != 0 {
            uart0::send(c);
        }
    }
}

