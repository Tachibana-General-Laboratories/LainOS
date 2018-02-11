use gpio::*;

use core::convert::AsRef;

//extern volatile unsigned int mbox[36];

//volatile unsigned int  __attribute__((aligned(16))) mbox[36];
/// a properly aligned buffer
pub static mut BUFFER: [Volatile<u32>; 36] = [Volatile::new(0); 36];

pub const REQUEST: u32 = 0;

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
pub const TAG_SETCLKRATE: u32 =     0x38002;
pub const TAG_LAST: u32 =           0;


const VIDEOCORE_MBOX: usize = MMIO_BASE + 0x0000B880;

const MBOX_READ: Mmio<u32> =       Mmio::new(VIDEOCORE_MBOX+0x00);
const MBOX_POLL: Mmio<u32> =       Mmio::new(VIDEOCORE_MBOX+0x10);
const MBOX_SENDER: Mmio<u32> =     Mmio::new(VIDEOCORE_MBOX+0x14);
const MBOX_STATUS: Mmio<u32> =     Mmio::new(VIDEOCORE_MBOX+0x18);
const MBOX_CONFIG: Mmio<u32> =     Mmio::new(VIDEOCORE_MBOX+0x1C);
const MBOX_WRITE: Mmio<u32> =      Mmio::new(VIDEOCORE_MBOX+0x20);

const MBOX_RESPONSE: u32 =   0x8000_0000;
const MBOX_FULL: u32 =       0x8000_0000;
const MBOX_EMPTY: u32 =      0x4000_0000;

/// Make a mailbox call. Returns 0 on failure, non-zero on success
pub fn call(ch: Channel) -> bool {
    unsafe {
        let ch = ch as u8;

        // wait until we can write to the mailbox
        while {
            unsafe { asm!("nop" :::: "volatile"); }
            MBOX_STATUS.read() & MBOX_FULL != 0
        } {}

        // write the address of our message to the mailbox with channel identifier
        //MBOX_WRITE.write((((unsigned int)((unsigned long)&mbox)&~0xF) | (ch & 0xF));
        let addr = (&BUFFER) as *const [Volatile<u32>; 36] as u32;
        MBOX_WRITE.write((addr & !0xF) | (ch & 0xF) as u32);

        // now wait for the response
        loop {
            // is there a response?
            while {
                asm!("nop" :::: "volatile");
                MBOX_STATUS.read() & MBOX_EMPTY != 0
            } {}

            let r = MBOX_READ.read();
            // is it a response to our message?
            if (r & 0xF) as u8 == ch && (r & !0xF) == addr {
                // is it a valid successful response?
                return BUFFER[1].read() == MBOX_RESPONSE;
            }
        }
        false
    }
}
