use core::fmt;

use core::intrinsics::abort;

#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_unwind(msg: fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\n\nPANIC {}:{}:", file, line);
    println!("    {}", msg);


    //TODO
    //unsafe { interrupt::stack_trace(); }

    println!("HALT");

    loop {
        // TODO
        // unsafe { interrupt::halt(); }
        unsafe { abort() }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    //println!("_Unwind_Resume");
    loop {
        // TODO
        // unsafe { interrupt::halt(); }
    }
}
