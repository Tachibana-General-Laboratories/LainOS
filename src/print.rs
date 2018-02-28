use pi::uart0;
use std::fmt::{self, Write};
use spin::Mutex;

pub struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut uart = uart0::Uart0::new();
        for c in s.chars() {
            // convert newline to carrige return + newline
            if c == '\n' {
                uart.send(b'\r');
            }
            uart.send(c as u8)
        }
        Ok(())
    }
}

pub static UART0: Mutex<Writer> = Mutex::new(Writer);

macro_rules! kprintln {
    () => (kprint!("\n"));
    ($fmt:expr) => (kprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (kprint!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! kprint {
    () => ();
    ($($arg:tt)*) => ({
        $crate::print::kprint(format_args!($($arg)*));
    });
}

pub fn kprint(args: fmt::Arguments) {
    UART0.lock().write_fmt(args).unwrap();
}
