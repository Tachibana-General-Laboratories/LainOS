use super::*;
use util;

use super::gpio::{self, Gpio};

use std::io;

use volatile::prelude::*;
use volatile::{ReadVolatile, Volatile, Reserved};

const PL011_UART: usize = IO_BASE + 0x201000;

// PL011 UART registers
#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    DR: Volatile<u8>,
    RSRECR: [Reserved<u32>; 5],
    FR: ReadVolatile<u8>,

    ILPR: Reserved<u32>,
    IBRD: Volatile<u32>,        // Integer Baud rate divisor
    FBRD: Volatile<u32>,        // Fractional Baud rate divisor
    LCRH: Volatile<u32>,        // Line Control register
    CR: Volatile<u32>,          // Control register
    IFLS: Reserved<u32>,        // Interupt FIFO Level Select Register
    IMSC: Volatile<u32>,        // Interupt Mask Set Clear Register
    RIS: Reserved<u32>,         // Raw Interupt Status Register
    MIS: Reserved<u32>,         // Masked Interupt Status Register
    ICR: Volatile<u32>,         // Interupt Clear Register
    DMACR: Reserved<u32>,       // DMA Control Register
    ITCR: Reserved<u32>,        // Test Control register
    ITIP: Reserved<u32>,        // Integration test input reg
    ITOP: Reserved<u32>,        // Integration test output reg
    TDR: Reserved<u32>,         // Test Data reg
}

pub struct Uart0 {
    registers: &'static mut Registers,
}

impl Uart0 {
    pub fn new() -> Self {
        unsafe { Self::from_addr(PL011_UART) }
    }

    pub unsafe fn from_addr(addr: usize) -> Self {
        Self {
            registers: &mut *(addr as *mut Registers),
        }
    }

    /// Set baud rate and characteristics (115200 8N1) and map to GPIO
    pub fn initialize(&mut self) {
        self.registers.CR.write(0);         // turn off UART0

        // set up clock for consistent divisor values
        let mut b = mbox::Mailbox::new();
        b[0].write(8 * 4);
        b[1].write(mbox::REQUEST);
        b[2].write(mbox::TAG_SETCLKRATE); // set clock rate
        b[3].write(12);
        b[4].write(8);
        b[5].write(2);           // UART clock
        b[6].write(4000000);     // 4Mhz
        b[7].write(mbox::TAG_LAST);
        b.call(mbox::Channel::PROP1).unwrap();

        for &pin in &[14, 15] {
            let mut pin = Gpio::new(pin).into_alt(gpio::Function::Alt0);
            pin.pull(gpio::Pull::Off);
        }

        self.registers.ICR.write(0x7FF);    // clear interrupts
        self.registers.IBRD.write(2);       // 115200 baud
        self.registers.FBRD.write(0xB);
        self.registers.LCRH.write(0b11<<5); // 8n1
        self.registers.CR.write(0x301);     // enable Tx, Rx, FIFO
    }

    /// Send a character
    pub fn send(&mut self, c: u8) {
        // wait until we can send
        util::nop_while(|| self.registers.FR.read() & 0x20 != 0);
        // write the character to the buffer
        self.registers.DR.write(c);
    }

    /// Receive a character
    pub fn receive(&self) -> u8 {
        // wait until something is in the buffer
        util::nop_while(|| self.registers.FR.read() & 0x10 != 0);
        // read it and return
        self.registers.DR.read()
    }
}

impl io::Read for Uart0 {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len();
        for b in buf {
            *b = self.receive();
        }
        Ok(len)
    }
}

impl io::Write for Uart0 {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        for &b in buf {
            self.send(b);
        }
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Send a character
pub fn send(c: u8) {
    Uart0::new().send(c)
}

/// Receive a character
pub fn receive() -> u8 {
    Uart0::new().receive()
}
