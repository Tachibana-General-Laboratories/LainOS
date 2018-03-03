use std::io;
use std::fmt;

//use pi::uart::MiniUart;
use pi::uart0::Uart0 as MiniUart;

use spin::Mutex;

/// A global singleton allowing read/write access to the console.
pub struct Console {
    inner: Option<MiniUart>
}

impl Console {
    /// Creates a new instance of `Console`.
    const fn new() -> Self {
        Self { inner: None }
    }

    /// Initializes the console if it's not already initialized.
    #[inline]
    fn initialize(&mut self) {
        let mut u = MiniUart::new();
        u.initialize();
        self.inner = Some(u);
    }

    /// Returns a mutable borrow to the inner `MiniUart`, initializing it as
    /// needed.
    fn inner(&mut self) -> &mut MiniUart {
        match self.inner {
            Some(ref mut uart) => uart,
            None => {
                self.initialize();
                self.inner.as_mut().unwrap()
            }
        }
    }

    /// Reads a byte from the UART device, blocking until a byte is available.
    pub fn read_byte(&mut self) -> u8 {
        self.inner().receive()
    }

    /// Writes the byte `byte` to the UART device.
    pub fn write_byte(&mut self, byte: u8) {
        self.inner().send(byte)
    }
}

impl io::Read for Console {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for c in buf.iter_mut() {
            *c = self.read_byte();
        }
        Ok(buf.len())
    }
}

impl io::Write for Console {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for &c in buf.iter() {
            self.write_byte(c);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut uart = self.inner();
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

/// Global `Console` singleton.
pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

/// Internal function called by the `kprint[ln]!` macros.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    #[cfg(not(test))]
    {
        use std::fmt::Write;
        let mut console = CONSOLE.lock();
        console.write_fmt(args).unwrap();
    }

    #[cfg(test)]
    { print!("{}", args); }
}

/// Like `println!`, but for kernel-space.
pub macro kprintln {
    () => (kprint!("\n")),
    ($fmt:expr) => (kprint!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (kprint!(concat!($fmt, "\n"), $($arg)*))
}

/// Like `print!`, but for kernel-space.
pub macro kprint($($arg:tt)*) {
    _print(format_args!($($arg)*))
}
