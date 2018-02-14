use super::IO_BASE;
use util;

use volatile::prelude::*;
use volatile::{Volatile, Reserved};
use core::ops::{Index, IndexMut};
use core::mem::transmute;

#[repr(align(16))]
/// a properly aligned buffer
struct Buffer {
    data: [u32; 36],
}

static mut BUFFER: Buffer = Buffer::new();

impl Buffer {
    const fn new() -> Self {
        Self { data: [0u32; 36] }
    }
    #[inline(always)]
    unsafe fn addr(&self) -> u32 {
        (&self.data) as *const [u32; 36] as u32
    }
}

// channels
#[repr(u8)]
pub enum Channel {
    POWER =   0, // Power management
    FB =      1, // Framebuffer
    VUART =   2, // Virtual UART
    VCHIQ =   3,
    LEDS =    4,
    BTNS =    5, // Buttons
    TOUCH =   6, // Touch screen
    COUNT =   7,
    PROP1 =   8, // Property tags (ARM -> VC)
    PROP2 =   9, // Property tags (VC -> ARM)
}

// tags
pub const TAG_GETSERIAL: u32 =      0x10004;
pub const TAG_SETPOWER: u32 =       0x28001;
pub const TAG_SETCLKRATE: u32 =     0x38002;
pub const TAG_LAST: u32 =           0;

const VIDEOCORE_MBOX: usize = IO_BASE + 0x0000B880;

pub const REQUEST: u32 = 0;
const RESPONSE: u32 =   0x8000_0000;
const FULL: u32 =       0x8000_0000;
const EMPTY: u32 =      0x4000_0000;

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    READ: Volatile<u32>,
    _a: Reserved<u32>,
    _b: Reserved<u32>,
    _c: Reserved<u32>,
    POLL: Volatile<u32>,
    SENDER: Volatile<u32>,
    STATUS: Volatile<u32>,
    CONFIG: Volatile<u32>,
    WRITE: Volatile<u32>,
}

pub struct Mailbox {
    registers: &'static mut Registers,
}

impl Mailbox {
    pub fn new() -> Self {
        Self {
            registers: unsafe { &mut *(VIDEOCORE_MBOX as *mut Registers) },
        }
    }

    fn status(&self) -> u32 {
        self.registers.STATUS.read()
    }
    fn read(&self) -> u32 {
        self.registers.READ.read()
    }
    fn write(&mut self, value: u32) {
        self.registers.WRITE.write(value)
    }

    /// Make a mailbox call. Returns 0 on failure, non-zero on success
    pub fn call(&mut self, ch: Channel)  -> Result<(), ()> {
        let ch = ch as u8;

        // wait until we can write to the mailbox
        util::nop_while(|| self.status() & FULL != 0);

        // write the address of our message to the mailbox with channel identifier
        let addr = unsafe { BUFFER.addr() };
        self.write((addr & !0xF) | (ch & 0xF) as u32);

        // now wait for the response
        loop {
            // is there a response?
            util::nop_while(|| self.status() & EMPTY != 0);

            let r = self.read();
            // is it a response to our message?
            if (r & 0xF) as u8 == ch && (r & !0xF) == addr {
                // is it a valid successful response?
                return if self[1].read() == RESPONSE {
                    Ok(())
                } else {
                    Err(())
                };
            }
        }
    }
}

impl Index<usize> for Mailbox {
    type Output = Volatile<u32>;

    #[inline(always)]
    fn index(&self, idx: usize) -> &Self::Output {
        debug_assert!(idx < 36);
        unsafe { transmute(&BUFFER.data[idx]) }
    }
}

impl IndexMut<usize> for Mailbox {
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        debug_assert!(idx < 36);
        unsafe { transmute(&mut BUFFER.data[idx]) }
    }
}

