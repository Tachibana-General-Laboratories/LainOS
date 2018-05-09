use core::fmt::{self, Write};
use pi::gpio::Gpio;

mod shell;
mod syscall;
use self::syscall::*;

#[no_mangle]
pub extern "C" fn el0_other() -> ! {
    println!("I just exit.");
    syscall_exit(127);
    loop {
        println!("I sleep two seconds.");
        syscall_sleep(2 * 1000).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn el0_shell() -> ! {
    println!("I just shell.");
    loop {
        let code = shell::shell("usr> ");
        println!("exit with code: {}", code);
    }
}

#[no_mangle]
pub extern "C" fn el0_init() -> ! {
    println!("im in a bear suite");
    unsafe { asm!("brk 1" :::: "volatile"); }
    println!("fuck you shit: {}", 555);
    unsafe { asm!("brk 2" :::: "volatile"); }

    unsafe {
        asm!("
            mov x1, #1
            mrs x1, TPIDR_EL0
            msr TPIDR_EL0, x1
            "
            :::: "volatile");
    }
    syscall_sleep(5 * 1000).unwrap();

    let mut led = Gpio::new(16).into_output();
    let mut motor = Gpio::new(20).into_output();
    let mut button = Gpio::new(18).into_input();

    let mut state = false;

    loop {
        let down = !button.level();
        //println!("loop with sleep: {}", down);

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
