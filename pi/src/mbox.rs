use common::{IO_BASE, spin_wait};

use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, Reserved};

/// enumerated mailbox channels
/// see https://github.com/raspberrypi/firmware/wiki/Mailboxes
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Channel {
    POWER =   0, // Mailbox Channel 0: Power Management Interface
    FB =      1, // Mailbox Channel 1: Frame Buffer
    VUART =   2, // Mailbox Channel 2: Virtual UART
    VCHIQ =   3, // Mailbox Channel 3: VCHIQ Interface
    LEDS =    4, // Mailbox Channel 4: LEDs Interface
    BUTTONS = 5, // Mailbox Channel 5: Buttons Interface
    TOUCH =   6, // Mailbox Channel 6: Touchscreen Interface
    COUNT =   7, // Mailbox Channel 7: Counter
    TAGS =    8, // Mailbox Channel 8: Tags (ARM to VC)
    GPU =     9, // Mailbox Channel 9: GPU (VC to ARM)
}

/// enumerated mailbox clock id
/// see https://github.com/raspberrypi/firmware/wiki/Mailboxes
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum ClockId {
    EMMC = 0x1,  // Mailbox Tag Channel EMMC clock ID
    UART = 0x2,  // Mailbox Tag Channel uart clock ID
    ARM = 0x3,   // Mailbox Tag Channel ARM clock ID
    CORE = 0x4,  // Mailbox Tag Channel SOC core clock ID
    V3D = 0x5,   // Mailbox Tag Channel V3D clock ID
    H264 = 0x6,  // Mailbox Tag Channel H264 clock ID
    ISP = 0x7,   // Mailbox Tag Channel ISP clock ID
    SDRAM = 0x8, // Mailbox Tag Channel SDRAM clock ID
    PIXEL = 0x9, // Mailbox Tag Channel PIXEL clock ID
    PWM = 0xA,   // Mailbox Tag Channel PWM clock ID
}

/// enumerated mailbox tag channel commands
/// see https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum Tag {
    // Videocore info commands
    GET_FIRMWARE_VERSION           = 0x00000001,            // Get firmware revision

    // Hardware info commands
    GET_BOARD_MODEL                = 0x00010001,            // Get board model
    GET_BOARD_REVISION             = 0x00010002,            // Get board revision
    GET_BOARD_MAC_ADDRESS          = 0x00010003,            // Get board MAC address
    GET_BOARD_SERIAL               = 0x00010004,            // Get board serial
    GET_ARM_MEMORY                 = 0x00010005,            // Get ARM memory
    GET_VC_MEMORY                  = 0x00010006,            // Get VC memory
    GET_CLOCKS                     = 0x00010007,            // Get clocks

    // Power commands
    GET_POWER_STATE                = 0x00020001,            // Get power state
    GET_TIMING                     = 0x00020002,            // Get timing
    SET_POWER_STATE                = 0x00028001,            // Set power state

    // GPIO commands
    GET_GET_GPIO_STATE             = 0x00030041,            // Get GPIO state
    SET_GPIO_STATE                 = 0x00038041,            // Set GPIO state

    // Clock commands
    GET_CLOCK_STATE                = 0x00030001,            // Get clock state
    GET_CLOCK_RATE                 = 0x00030002,            // Get clock rate
    GET_MAX_CLOCK_RATE             = 0x00030004,            // Get max clock rate
    GET_MIN_CLOCK_RATE             = 0x00030007,            // Get min clock rate
    GET_TURBO                      = 0x00030009,            // Get turbo

    SET_CLOCK_STATE                = 0x00038001,            // Set clock state
    SET_CLOCK_RATE                 = 0x00038002,            // Set clock rate
    SET_TURBO                      = 0x00038009,            // Set turbo

    // Voltage commands
    GET_VOLTAGE                    = 0x00030003,            // Get voltage
    GET_MAX_VOLTAGE                = 0x00030005,            // Get max voltage
    GET_MIN_VOLTAGE                = 0x00030008,            // Get min voltage

    SET_VOLTAGE                    = 0x00038003,            // Set voltage

    // Temperature commands
    GET_TEMPERATURE                = 0x00030006,            // Get temperature
    GET_MAX_TEMPERATURE            = 0x0003000A,            // Get max temperature

    // Memory commands
    ALLOCATE_MEMORY                = 0x0003000C,            // Allocate Memory
    LOCK_MEMORY                    = 0x0003000D,            // Lock memory
    UNLOCK_MEMORY                  = 0x0003000E,            // Unlock memory
    RELEASE_MEMORY                 = 0x0003000F,            // Release Memory

    // Execute code commands
    EXECUTE_CODE                   = 0x00030010,            // Execute code

    // QPU control commands
    EXECUTE_QPU                    = 0x00030011,            // Execute code on QPU
    ENABLE_QPU                     = 0x00030012,            // QPU enable

    // Displaymax commands
    GET_DISPMANX_HANDLE            = 0x00030014,            // Get displaymax handle
    GET_EDID_BLOCK                 = 0x00030020,            // Get HDMI EDID block

    // SD Card commands
    GET_SDHOST_CLOCK               = 0x00030042,            // Get SD Card EMCC clock
    SET_SDHOST_CLOCK               = 0x00038042,            // Set SD Card EMCC clock

    // Framebuffer commands
    ALLOCATE_FRAMEBUFFER           = 0x00040001,            // Allocate Framebuffer address
    BLANK_SCREEN                   = 0x00040002,            // Blank screen
    GET_PHYSICAL_WIDTH_HEIGHT      = 0x00040003,            // Get physical screen width/height
    GET_VIRTUAL_WIDTH_HEIGHT       = 0x00040004,            // Get virtual screen width/height
    GET_COLOUR_DEPTH               = 0x00040005,            // Get screen colour depth
    GET_PIXEL_ORDER                = 0x00040006,            // Get screen pixel order
    GET_ALPHA_MODE                 = 0x00040007,            // Get screen alpha mode
    GET_PITCH                      = 0x00040008,            // Get screen line to line pitch
    GET_VIRTUAL_OFFSET             = 0x00040009,            // Get screen virtual offset
    GET_OVERSCAN                   = 0x0004000A,            // Get screen overscan value
    GET_PALETTE                    = 0x0004000B,            // Get screen palette

    RELEASE_FRAMEBUFFER            = 0x00048001,            // Release Framebuffer address
    SET_PHYSICAL_WIDTH_HEIGHT      = 0x00048003,            // Set physical screen width/heigh
    SET_VIRTUAL_WIDTH_HEIGHT       = 0x00048004,            // Set virtual screen width/height
    SET_COLOUR_DEPTH               = 0x00048005,            // Set screen colour depth
    SET_PIXEL_ORDER                = 0x00048006,            // Set screen pixel order
    SET_ALPHA_MODE                 = 0x00048007,            // Set screen alpha mode
    SET_VIRTUAL_OFFSET             = 0x00048009,            // Set screen virtual offset
    SET_OVERSCAN                   = 0x0004800A,            // Set screen overscan value
    SET_PALETTE                    = 0x0004800B,            // Set screen palette
    SET_VSYNC                      = 0x0004800E,            // Set screen VSync
    SET_BACKLIGHT                  = 0x0004800F,            // Set screen backlight

    // VCHIQ command
    VCHIQ_INIT                     = 0x00048010,            // Enable VCHIQ

    // Config commands
    GET_COMMAND_LINE               = 0x00050001,            // Get command line 

    // Shared resource management commands
    GET_DMA_CHANNELS               = 0x00060001,            // Get DMA channels

    // Cursor commands
    SET_CURSOR_INFO                = 0x00008010,            // Set cursor info
    SET_CURSOR_STATE               = 0x00008011,            // Set cursor state
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    READ0: ReadVolatile<u32>,                    // 0x00         Read data from VC to ARM
    UNUSED: [Reserved<u32>; 3],                  // 0x04-0x0F
    PEEK0: Volatile<u32>,                        // 0x10
    SENDER0: Volatile<u32>,                      // 0x14
    STATUS0: Volatile<u32>,                      // 0x18         Status of VC to ARM
    CONFIG0: Volatile<u32>,                      // 0x1C
    WRITE1: Volatile<u32>,                       // 0x20         Write data from ARM to VC
    UNUSED2: [Reserved<u32>; 3],                 // 0x24-0x2F
    PEEK1: Volatile<u32>,                        // 0x30
    SENDER1: Volatile<u32>,                      // 0x34
    STATUS1: Volatile<u32>,                      // 0x38         Status of ARM to VC
    CONFIG1: Volatile<u32>,                      // 0x3C
}


/// ARM bus address to GPU bus address
#[inline(always)]
pub fn arm2gpu(addr: usize) -> usize {
    addr | 0xC000_0000
}

/// GPU bus address to ARM bus address
#[inline(always)]
pub fn gpu2arm(addr: usize) -> usize {
    addr & !0xC000_0000
}

/// Mailbox Status Register: Mailbox Empty
const MAIL_EMPTY: u32 = 0x40000000;
/// Mailbox Status Register: Mailbox Full
const MAIL_FULL: u32 =  0x80000000;

const MAIL_RESPONSE: u32 = 0x8000_0000;


// tags
pub const TAG_GETSERIAL: u32 =      0x10004;
pub const TAG_SETPOWER: u32 =       0x28001;
pub const TAG_SETCLKRATE: u32 =     0x38002;
pub const TAG_LAST: u32 =           0;

const VIDEOCORE_MBOX: usize = IO_BASE + 0x0000B880;

const REQUEST: u32 = 0;

pub struct Mailbox {
    registers: &'static mut Registers,
}

impl Mailbox {
    pub fn new() -> Self {
        Self {
            registers: unsafe { &mut *(VIDEOCORE_MBOX as *mut Registers) },
        }
    }

    /// This will post and execute the given variadic data onto the tags channel
    /// on the mailbox system. You must provide the correct number of response
    /// uint32_t variables and a pointer to the response buffer. You nominate the
    /// number of data uint32_t for the call and fill the variadic data in. If you
    /// do not want the response data back the use NULL for response_buf pointer.
    /// RETURN: True for success and the response data will be set with data
    ///         False for failure and the response buffer is untouched.
    pub fn tag_message(&mut self, request: &[u32]) -> Option<[u32; 36]> {
        assert!(request.len() <= 36 - 3);

        let mut message = Buffer::new();
        // Size of message needed
        message.data[0] = (request.len() as u32 + 3) * 4;
        // Set end pointer to zero
        message.data[request.len() + 2] = 0;
        // Zero response message
        message.data[1] = REQUEST;
        message.data[2..request.len() + 2].copy_from_slice(request);

        // Write message to mailbox
        self.write(Channel::TAGS, arm2gpu(message.addr() as usize) as u32);
        // Wait for write response
        self.read(Channel::TAGS);
        if message.data[1] == MAIL_RESPONSE {
            Some(message.data)
        } else {
            None
        }
    }

    /// This will execute the sending of the given data block message thru the
    /// mailbox system on the given channel.
    pub fn write(&mut self, channel: Channel, mut message: u32) {
        // Make sure 4 low channel bits are clear
        message &= !0xF;
        // OR the channel bits to the value
        message |= channel as u32;
        // Read mailbox1 status from GPU
        // Make sure arm mailbox is not full
        spin_wait(|| self.registers.STATUS1.read() & MAIL_FULL != 0);
        // Write value to mailbox
        self.registers.WRITE1.write(message);
    }

    /// This will read any pending data on the mailbox system on the given channel.
    pub fn read(&self, channel: Channel) -> u32 {
        let mut value;
        let channel = channel as u32;
        while {
            // Read mailbox0 status
            // Wait for data in mailbox
            spin_wait(|| (self.registers.STATUS0.read() & MAIL_EMPTY) != 0);
            // Read the mailbox
            value = self.registers.READ0.read();
            // We have response back
            (value & 0xF) != channel
        } {}
        // Lower 4 low channel bits are not part of message
        value & !0xF
    }
}

/// a properly aligned buffer
#[repr(align(16))]
struct Buffer {
    data: [u32; 36],
}

impl Buffer {
    const fn new() -> Self {
        Self { data: [0u32; 36] }
    }
    #[inline(always)]
    fn addr(&self) -> u32 {
        (&self.data) as *const [u32; 36] as u32
    }
}
