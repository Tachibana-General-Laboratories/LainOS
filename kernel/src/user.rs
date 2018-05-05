use core::fmt::{self, Write};
use pi::gpio::Gpio;

use traps::Error as SysErr;

struct Stdout;

impl Write for Stdout {
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

#[no_mangle]
pub extern "C" fn el0_other() -> ! {
    loop {
        println!("i'm 2");
        syscall_sleep(10 * 1000).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn el0_init() -> ! {
    println!("im in a bear suite");
    unsafe { asm!("brk 1" :::: "volatile"); }
    println!("fuck you shit: {}", 555);
    unsafe { asm!("brk 2" :::: "volatile"); }

    let mut led = Gpio::new(16).into_output();
    let mut motor = Gpio::new(20).into_output();
    let mut button = Gpio::new(18).into_input();

    let mut state = false;

    loop {
        let down = !button.level();
        println!("loop with sleep: {}", down);

        state = down && !state;

        if state {
            motor.set();
            led.set();
            syscall_sleep(50).unwrap();
            motor.clear();
            led.clear();
            syscall_sleep(50).unwrap();
        } else {
            motor.clear();
            led.clear();
        }
    }

    //shell::shell("user0> ")
}


fn syscall_print(s: &str) -> Result<(), SysErr> {
    let error: u64;
    unsafe {
        asm!("mov x0, $1
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

fn syscall_sleep(ms: u32) -> Result<(), SysErr> {
    let error: u64;
    unsafe {
        asm!("mov x0, $1
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


