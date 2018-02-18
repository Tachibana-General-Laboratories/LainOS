use std::fmt;

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments, file: &'static str, line: u32, col: u32) -> ! {
    println!("\n\n");
    println!("---------- KERNEL PANIC ----------");
    println!("");
    println!("FILE: {}", file);
    println!("LINE: {}", line);
    println!(" COL: {}", col);
    println!("");
    println!("{}", msg);

    loop { unsafe { asm!("wfe") } }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    println!("_Unwind_Resume");
    loop { }
}
