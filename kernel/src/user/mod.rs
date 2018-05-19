mod shell;
mod syscall;
use self::syscall::*;

#[no_mangle]
pub extern "C" fn el0_other() -> ! {
    println!("I just exit.");
    //loop {}
    syscall_exit(127);
}

#[no_mangle]
pub extern "C" fn el0_shell() -> ! {
    println!("I sleep 50ms.");
    syscall_sleep(50).unwrap();

    println!("And then I just shell.");
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

    //loop {}

    println!("test GPIO");

    use pi::gpio::{Gpio, GPIO_BASE};
    use pi::common::IO_BASE_RAW;

    let mut r = unsafe { Gpio::new_from(IO_BASE_RAW + GPIO_BASE, 21).into_output() };
    let mut g = unsafe { Gpio::new_from(IO_BASE_RAW + GPIO_BASE, 20).into_output() };
    let mut b = unsafe { Gpio::new_from(IO_BASE_RAW + GPIO_BASE, 19).into_output() };

    let mut motor = unsafe { Gpio::new_from(IO_BASE_RAW + GPIO_BASE, 26).into_output() };
    let mut btn = unsafe { Gpio::new_from(IO_BASE_RAW + GPIO_BASE, 16).into_input() };

    let mut led = 0;

    loop {
        if led & 1 != 0 { r.set() } else { r.clear() }
        if led & 2 != 0 { g.set() } else { g.clear() }
        if led & 4 != 0 { b.set() } else { b.clear() }
        if !btn.level() {
            led += 1;
            motor.set();
            syscall_sleep(5).unwrap();
            motor.clear();
            syscall_sleep(100).unwrap();
        }
    }
}
