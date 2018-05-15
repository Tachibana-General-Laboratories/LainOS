use core::fmt;

use sys::volatile::prelude::*;
use sys::volatile::{Volatile, ReadVolatile, Reserved};

use timer;
use common::IO_BASE;
use gpio::{Gpio, Function};

/// The base address for the `MU` registers.
pub const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
pub const AUX_ENABLES: usize = IO_BASE + 0x215004;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IO: Volatile<u8>,
    _r0: [Reserved<u8>; 3],
    IER: Volatile<u8>,
    _r1: [Reserved<u8>; 3],
    IIR: Volatile<u8>,
    _r2: [Reserved<u8>; 3],
    LCR: Volatile<u8>,
    _r3: [Reserved<u8>; 3],
    MCR: Volatile<u8>,
    _r4: [Reserved<u8>; 3],
    LSR: ReadVolatile<u8>,
    _r5: [Reserved<u8>; 3],
    MSR: Volatile<u8>,
    _r6: [Reserved<u8>; 3],
    SCRATCH: Volatile<u8>,
    _r7: [Reserved<u8>; 3],
    CNTL: Volatile<u8>,
    _r8: [Reserved<u8>; 3],
    STAT: Volatile<u32>,
    BAUD: Volatile<u16>,
    _r9: [Reserved<u8>; 2],
}

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<u32>,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> Self {
        unsafe { Self::new_from(MU_REG_BASE, AUX_ENABLES) }
    }

    pub unsafe fn new_from(base: usize, aux: usize) -> Self {
        // Enable the mini UART as an auxiliary device.
        let aux = aux as *mut Volatile<u8>;
        (*aux).or_mask(1);
        let registers = &mut *(base as *mut Registers);

        // setting the data size to 8 bits
        registers.LCR.write(0b11);

        // setting the BAUD rate to ~115200 (baud divider of 270)
        registers.BAUD.write(270);

        // setting GPIO pins 14 and 15 to alternative function 5 (TXD1/RDXD1)
        Gpio::new(14).into_alt(Function::Alt5);
        Gpio::new(15).into_alt(Function::Alt5);

        // enable Tx, Rx
        registers.CNTL.write(0b11);

        Self { registers, timeout: None }
    }

    /// Set the read timeout to `milliseconds` milliseconds.
    pub fn set_read_timeout(&mut self, milliseconds: u32) {
        self.timeout = Some(milliseconds);
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        const MASK: u8 = LsrStatus::TxAvailable as u8;
        while {
            unsafe { asm!("nop" :::: "volatile"); }
            self.registers.LSR.read() & MASK == 0
        } {}
        self.registers.IO.write(byte);
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        const MASK: u8 = LsrStatus::DataReady as u8;
        self.registers.LSR.read() & MASK != 0
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        if let Some(timeout) = self.timeout {
            let end_time = timer::current_time() + 1000 * timeout as u64;
            loop {
                if self.has_byte() {
                    return Ok(());
                }
                if timer::current_time() >= end_time {
                    return Err(())
                }
            }
        } else {
            while {
                unsafe { asm!("nop" :::: "volatile"); }
                !self.has_byte()
            } {}
            Ok(())
        }
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        while !self.has_byte() {
            unsafe { asm!("nop" :::: "volatile"); }
        }
        self.registers.IO.read()
    }

    pub fn write_buf(&mut self, buf: &[u8]) {
        for &b in buf {
            self.write_byte(b);
        }
    }

    pub fn read_buf(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.wait_for_byte().is_err() {
            return None;
        }
        let mut count = 0;
        for b in buf.iter_mut() {
            if !self.has_byte() {
                break;
            }
            *b = self.read_byte();
            count += 1;
        }
        Some(count)
    }
}

// FIXME: Implement `fmt::Write` for `MiniUart`. A b'\r' byte should be written
// before writing any b'\n' byte.
impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for c in s.chars() {
            if c == '\n' {
                self.write_byte(b'\r');
                self.write_byte(b'\n');
                continue;
            }

            if c.is_ascii() {
                self.write_byte(c as u8);
            } else {
                let mut buf = [0; 16];
                for &c in c.encode_utf8(&mut buf).as_bytes() {
                    self.write_byte(c);
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
mod uart_io {
    use std::io;
    use super::MiniUart;

    // FIXME: Implement `io::Read` and `io::Write` for `MiniUart`.
    //
    // The `io::Read::read()` implementation must respect the read timeout by
    // waiting at most that time for the _first byte_. It should not wait for
    // any additional bytes but _should_ read as many bytes as possible. If the
    // read times out, an error of kind `TimedOut` should be returned.
    impl io::Read for MiniUart {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.wait_for_byte().is_err() {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "uart time out"));
            }
            let mut count = 0;
            for b in buf.iter_mut() {
                if !self.has_byte() {
                    break;
                }
                *b = self.read_byte();
                count += 1;
            }
            Ok(count)
        }
    }

    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.
    impl io::Write for MiniUart {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            for &b in buf {
                self.write_byte(b);
            }
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
