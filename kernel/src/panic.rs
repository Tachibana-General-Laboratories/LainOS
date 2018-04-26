use console::{kprint, kprintln};

use core::fmt;

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments, file: &'static str, line: u32, col: u32) -> ! {
    kprintln!("\n\n");
    kprintln!("---------- KERNEL PANIC ----------");
    kprintln!("");
    kprintln!("FILE: {}", file);
    kprintln!("LINE: {}", line);
    kprintln!(" COL: {}", col);
    kprintln!("");
    kprintln!("{}", msg);

    loop { unsafe { asm!("wfe") } }
}
