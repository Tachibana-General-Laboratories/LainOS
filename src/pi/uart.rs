use core::fmt;

use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, Reserved};

use timer;
use common::IO_BASE;
use gpio::{Gpio, Function};

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

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
    IER: Volatile<u8>,
    IIR: Volatile<u8>,
    LCR: Volatile<u8>,
    MCR: Volatile<u8>,
    LSR: Volatile<u8>,
    MSR: Volatile<u8>,
    SCRATCH: Volatile<u8>,
    CNTL: Volatile<u8>,
    STAT: Volatile<u32>,
    BAUD: Volatile<u16>,
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
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        // FIXME: Implement remaining mini UART initialization.
        unimplemented!()
    }

    /// Set the read timeout to `milliseconds` milliseconds.
    pub fn set_read_timeout(&mut self, milliseconds: u32) {
        unimplemented!()
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        unimplemented!()
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        unimplemented!()
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
        unimplemented!()
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        unimplemented!()
    }
}

// FIXME: Implement `fmt::Write` for `MiniUart`. A b'\r' byte should be written
// before writing any b'\n' byte.

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
    //
    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.
}


/*
// Auxilary mini UART registers
const UART_BASE: usize = 0x3F201000;
//0x00215004

pub const AUX_ENABLE: Mmio<u8> =      Mmio::new(UART_BASE + 0x04);

pub const AUX_MU_IO: Mmio<u8> =       Mmio::new(UART_BASE + 0x40);
pub const AUX_MU_IER: Mmio<u8> =      Mmio::new(UART_BASE + 0x44);
pub const AUX_MU_IIR: Mmio<u8> =      Mmio::new(UART_BASE + 0x48);
pub const AUX_MU_LCR: Mmio<u8> =      Mmio::new(UART_BASE + 0x4C);
pub const AUX_MU_MCR: Mmio<u8> =      Mmio::new(UART_BASE + 0x50);
pub const AUX_MU_LSR: Mmio<u8> =      Mmio::new(UART_BASE + 0x54);
pub const AUX_MU_MSR: Mmio<u8> =      Mmio::new(UART_BASE + 0x58);
pub const AUX_MU_SCRATCH: Mmio<u8> =  Mmio::new(UART_BASE + 0x5C);
pub const AUX_MU_CNTL: Mmio<u8> =     Mmio::new(UART_BASE + 0x60);
pub const AUX_MU_STAT: Mmio<u32> =    Mmio::new(UART_BASE + 0x64);
pub const AUX_MU_BAUD: Mmio<u16> =    Mmio::new(UART_BASE + 0x68);

/// Set baud rate and characteristics (115200 8N1) and map to GPIO
pub fn init() {
    // initialize UART
    AUX_ENABLE.write(AUX_ENABLE.read() | 1);       // enable UART1, AUX mini uart
    AUX_MU_IER.write(0);
    AUX_MU_CNTL.write(0);
    AUX_MU_LCR.write(3);       // 8 bits
    AUX_MU_MCR.write(0);
    AUX_MU_IER.write(0);
    AUX_MU_IIR.write(0xc6);    // disable interrupts
    AUX_MU_BAUD.write(270);    // 115200 baud

    // map UART1 to GPIO pins
    let mut r = GPFSEL1.read();
    r &= !(7<<12 | 7<<15); // gpio14, gpio15
    r |= 2<<12 | 2<<15 ;    // alt5
    GPFSEL1.write(r);

    GPPUD.write(0);            // enable pins 14 and 15
    nop_delay(150);

    GPPUDCLK0.write(1<<14 | 1<<15);
    nop_delay(150);

    GPPUDCLK0.write(0);        // flush GPIO setup
    AUX_MU_CNTL.write(3);      // enable Tx, Rx
}

/// Send a character
pub fn send(c: u8) {
    // wait until we can send
    while {
        unsafe { asm!("nop" ::: "volatile"); }
        AUX_MU_LSR.read() & 0x20 != 0
    } {}
    // write the character to the buffer
    AUX_MU_IO.write(c);
}

/// Receive a character
pub fn receive() -> u8 {
    // wait until something is in the buffer
    while {
        unsafe { asm!("nop" ::: "volatile"); }
        AUX_MU_LSR.read() & 0x01 != 0
    } {}
    // read it and return 
    AUX_MU_IO.read()
}

/*
/// Receive a character
pub fn getc() -> u8 {
    // wait until something is in the buffer
    do{asm volatile("nop");}while(!(*AUX_MU_LSR&0x01));
    // read it and return 
    let r = (char)(*AUX_MU_IO) as u8;
    // convert carrige return to newline
    if r == '\r' {
        '
    }
    return r=='\r'?'\n':r;
}
*/
*/
