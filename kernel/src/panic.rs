use pi::uart::MiniUart;
use core::fmt::{self, Write, Arguments};

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: Arguments, file: &'static str, line: u32, col: u32) -> ! {
    let mut uart = MiniUart::new();

    let _ = uart.write_str("\n\n---------- KERNEL PANIC ----------\n");
    let _ = uart.write_fmt(format_args!("FILE: {}\nLINE: {}\n COL: {}\n", file, line, col));
    let _ = uart.write_fmt(format_args!("{}", msg));

    loop { unsafe { asm!("wfe") } }
}
