use std::fmt;

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

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    kprintln!("_Unwind_Resume");
    loop { }
}
