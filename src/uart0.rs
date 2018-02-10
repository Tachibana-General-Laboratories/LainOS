use gpio::*;

// PL011 UART registers
pub const UART0_DR: Mmio<u8> =        Mmio::new(MMIO_BASE+0x00201000);
pub const UART0_FR: Mmio<u8> =        Mmio::new(MMIO_BASE+0x00201018);

pub const UART0_IBRD: Mmio<u32> =      Mmio::new(MMIO_BASE+0x00201024);
pub const UART0_FBRD: Mmio<u32> =      Mmio::new(MMIO_BASE+0x00201028);
pub const UART0_LCRH: Mmio<u32> =      Mmio::new(MMIO_BASE+0x0020102C);
pub const UART0_CR: Mmio<u32> =        Mmio::new(MMIO_BASE+0x00201030);
pub const UART0_IMSC: Mmio<u32> =      Mmio::new(MMIO_BASE+0x00201038);
pub const UART0_ICR: Mmio<u32> =       Mmio::new(MMIO_BASE+0x00201044);

fn nop_delay(r: u32) {
    for _ in 0..r {
        unsafe { asm!("nop" ::: "volatile"); }
    }
}

/// Set baud rate and characteristics (115200 8N1) and map to GPIO
pub fn init() {
    // initialize UART
    UART0_CR.write(0);         // turn off UART0

    /*
    // set up clock for consistent divisor values
    mbox[0] = 8*4;
    mbox[1] = MBOX_REQUEST;
    mbox[2] = MBOX_TAG_SETCLKRATE; // set clock rate
    mbox[3] = 12;
    mbox[4] = 8;
    mbox[5] = 2;           // UART clock
    mbox[6] = 4000000;     // 4Mhz
    mbox[7] = MBOX_TAG_LAST;
    mbox_call(MBOX_CH_PROP);
    */

    /* map UART0 to GPIO pins */
    let mut r = GPFSEL1.read();
    r &= !(7<<12 | 7<<15); // gpio14, gpio15
    r |= 4<<12 | 4<<15;    // alt0
    GPFSEL1.write(r);

    GPPUD.write(0);            // enable pins 14 and 15
    nop_delay(150);
    GPPUDCLK0.write(1<<14 | 1<<15);
    nop_delay(150);
    GPPUDCLK0.write(0);        // flush GPIO setup

    UART0_ICR.write(0x7FF);    // clear interrupts
    UART0_IBRD.write(2);       // 115200 baud
    UART0_FBRD.write(0xB);
    UART0_LCRH.write(0b11<<5); // 8n1
    UART0_CR.write(0x301);     // enable Tx, Rx, FIFO
}

/// Send a character
pub fn send(c: u8) {
    // wait until we can send
    while {
        unsafe { asm!("nop" ::: "volatile"); }
        UART0_FR.read() & 0x20 != 0
    } {}
    // write the character to the buffer
    UART0_DR.write(c);
}

/// Receive a character
pub fn receive() -> u8 {
    // wait until something is in the buffer
    while {
        unsafe { asm!("nop" ::: "volatile"); }
        UART0_FR.read() & 0x10 != 0
    } {}
    // read it and return
    UART0_DR.read()
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

/// Display a string
pub fn puts(msg: &str) {
    for c in msg.chars() {
        // convert newline to carrige return + newline
        if c == '\n' {
            send(b'\r');
        }
        send(c as u8)
    }
}
