mod shell;
mod syscall;
use self::syscall::*;

#[no_mangle]
pub extern "C" fn el0_other() -> ! {
    println!("I just exit.");
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

    loop {}

    use pi::gpio::Gpio;
    let mut led = Gpio::new(16).into_output();
    let mut motor = Gpio::new(20).into_output();
    let mut button = Gpio::new(18).into_input();

    let mut state = false;

    loop {
        state = !button.level() && !state;

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
}
