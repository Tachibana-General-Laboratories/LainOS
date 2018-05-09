use core::fmt::{self, Write};
use traps::Error as SysErr;

pub struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        syscall_print(s).unwrap();
        Ok(())
    }
}

pub macro println {
    () => (print!("\n")),
    ($fmt:expr) => (print!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*))
}

/// Like `print!`, but for kernel-space.
pub macro print($($arg:tt)*) {
    Stdout.write_fmt(format_args!($($arg)*)).unwrap()
}

pub fn write_buf(buf: &[u8]) {
    //CONSOLE.lock().unwrap().write_buf(buf)
    let s = unsafe { ::core::str::from_utf8_unchecked(buf) };
    Stdout.write_str(s).unwrap()
}

pub fn syscall_print(s: &str) -> Result<(), SysErr> {
    let error: u64;
    unsafe {
        asm!("
            mov x0, $1
            mov x1, $2
            svc 2
            mov $0, x7
            "
            : "=r"(error)
            : "r"(s.as_ptr()), "r"(s.len())
            : "x0", "x1", "x7"
            : "volatile")
    }
    if error == 0 {
        Ok(())
    } else {
        Err(SysErr::from(error))
    }
}

pub fn syscall_sleep(ms: u32) -> Result<(), SysErr> {
    let error: u64;
    unsafe {
        asm!("
            mov x0, $1
            svc 1
            mov $0, x7
            "
            : "=r"(error)
            : "r"(ms)
            : "x0", "x7"
            : "volatile");
    }
    if error == 0 {
        Ok(())
    } else {
        Err(SysErr::from(error))
    }
}

pub fn syscall_exit(code: u32) -> ! {
    unsafe {
        asm!("
            mov x0, $0
            svc 3
            "
            :
            : "r"(code)
            : "x0"
            : "volatile");
    }
    unreachable!("syscall_exit");
}

pub fn syscall_read_byte() -> Result<u8, SysErr> {
    let error: u64;
    let byte: u64;
    unsafe {
        asm!("
            svc 4
            mov $0, x7
            mov $1, x0
            "
            : "=r"(error), "=r"(byte)
            :
            : "x0", "x7"
            : "volatile");
    }
    if error == 0 {
        Ok(byte as u8)
    } else {
        Err(SysErr::from(error))
    }
}
