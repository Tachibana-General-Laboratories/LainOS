#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![feature(core_intrinsics)]
#![feature(const_fn)]

#![no_std]
#![no_builtins]

use core::intrinsics::{volatile_load, volatile_store};

pub mod externs;
pub mod panic;

pub mod gpio;
pub mod uart;

#[no_mangle]
pub extern "C" fn kernel_main(_r0: u32, _r1: u32, _atags: u32) {
    uart::init();
    uart::puts("Hello Rust Kernel world!");
    loop {
        let c = uart::receive();
        if c != 0 {
            uart::send(c);
        }
    }

    loop {}
}
