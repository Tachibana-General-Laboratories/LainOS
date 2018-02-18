#![allow(unused_mut)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_parens)]
#![allow(unused_assignments)]

use util::wait_msec;
use util::wait_cycles;

use pi::gpio;
use pi::IO_BASE;
use volatile::prelude::*;
use volatile::{Volatile, Reserved};

pub type c_int = i32;
pub type c_uint = u32;
pub type c_long = u64;

fn uart_puts(s: &str) {
    print!("{}", s);
}

fn uart_send(s: char) {
    print!("{}", s as u8);
}

pub enum Error {
    General,       // General non specific SD error
    Timeout,       // SD Timeout error
    Busy,          // SD Card is busy
    NoRespond,     // SD Card did not respond
    Reset,         // SD Card did not reset
    Clock,         // SD Card clock change failed
    Voltage,       // SD Card does not support requested voltage
    AppCommand,    // SD Card app command failed						
    CardAbsent,    // SD Card not present
    Read,
    MountFail,
}

bitflags! {
    struct InterruptFlags: u32 {
        const ACMD_ERR  = 0x01000000;
        const DEND_ERR  = 0x00400000;
        const DCRC_ERR  = 0x00200000;
        const DTO_ERR   = 0x00100000;
        const CBAD_ERR  = 0x00080000;
        const CEND_ERR  = 0x00040000;
        const CCRC_ERR  = 0x00020000;
        const CTO_ERR   = 0x00010000;
        const ERR       = 0x00008000;

        const ENDBOOT   = 0x00004000;
        const BOOTACK   = 0x00002000;
        const RETUNE    = 0x00001000;
        const CARD      = 0x00000100;
        const READ_RDY  = 0x00000020;
        const WRITE_RDY = 0x00000010;
        const BLOCK_GAP = 0x00000004;
        const DATA_DONE = 0x00000002;
        const CMD_DONE  = 0x00000001;

        const ERROR_MASK = Self::CBAD_ERR.bits
                         | Self::CCRC_ERR.bits
                         | Self::CCRC_ERR.bits
                         | Self::DTO_ERR.bits
                         | Self::DCRC_ERR.bits
                         | Self::DEND_ERR.bits
                         | Self::ERR.bits
                         | Self::ACMD_ERR.bits;

        const ALL_MASK = Self::CMD_DONE.bits
                       | Self::DATA_DONE.bits
                       | Self::READ_RDY.bits
                       | Self::WRITE_RDY.bits
                       | Self::ERROR_MASK.bits;
    }
}

impl InterruptFlags {
    pub fn is_err(&self) -> bool {
        self.intersects(Self::ERROR_MASK)
    }
}

struct Registers {
    ARG2: Volatile<u32>,
    BLKSIZECNT: Volatile<u32>,
    ARG1: Volatile<u32>,
    CMDTM: Volatile<u32>,
    RESP0: Volatile<u32>,
    RESP1: Volatile<u32>,
    RESP2: Volatile<u32>,
    RESP3: Volatile<u32>,
    DATA: Volatile<u32>,
    STATUS: Volatile<u32>,
    CONTROL0: Volatile<u32>,
    CONTROL1: Volatile<u32>,
    INTERRUPT: Volatile<InterruptFlags>,
    INT_MASK: Volatile<u32>,
    INT_EN: Volatile<u32>,
    CONTROL2: Volatile<u32>,
    _a: [Reserved<u32>; 4], // 0x40, 44, 48, 4c
    FORCE_IRPT: Volatile<u32>,
    _b: [Reserved<u32>; 7], // 0x54, 58, 5c, 60, 64, 68, 6c
    BOOT_TIMEOUT: Volatile<u32>,
    DBG_SEL: Volatile<u32>,
    _c: [Reserved<u32>; 2], // 0x78, 7c
    EXRDFIFO_CFG: Volatile<u32>,
    EXRDFIFO_EN: Volatile<u32>,
    TUNE_STEP: Volatile<u32>,
    TUNE_STEPS_STD: Volatile<u32>,
    TUNE_STEPS_DDR: Volatile<u32>,
    _d: [Reserved<u32>; 23], // 0x94, 98, 9c, ...
    SPI_INT_SPT: Volatile<u32>,
    _e: [Reserved<u32>; 2], //
    SLOTISR_VER: Volatile<u32>,
}

struct SD {
    reg: &'static mut Registers,
}

static mut EMMC: *mut Registers = (IO_BASE + 0x300000) as *mut Registers;

static mut sd_scr: [c_long; 2] = [0, 0];
static mut sd_ocr: c_long = 0;
static mut sd_rca: c_long = 0;
static mut sd_err: c_long = 0;
static mut sd_hv: c_long = 0;

const INT_ERROR_MASK: c_uint = 25067520;
const SD_ERROR: c_int = -2;
const SD_OK: c_int = 0;

const INT_CMD_TIMEOUT: c_uint = 65536;
const INT_DATA_TIMEOUT: c_uint = 1048576;
const SD_TIMEOUT: c_int = -1;

const CMD_NEED_APP: c_uint = 2147483648;
const CMD_APP_CMD: u32 = 922746880;
const CMD_RSPNS_48: u32 = 131072;
const SR_CMD_INHIBIT: c_uint = 1;
const CMD_SEND_OP_COND: c_uint = 687996928 | 2147483648;
const CMD_SEND_IF_COND: c_uint = 134348800;
const INT_CMD_DONE: c_uint = 1;
const CMD_GO_IDLE: c_uint = 0;
const SR_APP_CMD: u64 = 32;
const CMD_ALL_SEND_CID: c_uint = 33619968;
const CMD_SEND_REL_ADDR: c_uint = 50462720;
const CMD_ERRORS_MASK: u64 = 4294557700;
const CMD_RCA_MASK: u64 = 4294901760;

const SR_DAT_INHIBIT: c_uint = 2;
const SCR_SUPP_CCS: c_long = 1;
const SCR_SUPP_SET_BLKCNT: c_long = 33554432;
const CMD_SET_BLOCKCNT: c_uint = 386007040;
const CMD_READ_SINGLE: u32 = 287440912;
const CMD_READ_MULTI: u32 = 304218162;
const INT_READ_RDY: c_uint = 32;
const CMD_STOP_TRANS: c_uint = 201523200;

const C1_CLK_EN: u32 = 4;
const HOST_SPEC_V2: c_long = 1;
const C1_CLK_STABLE: c_uint = 2;

// GPIO_CD
const GPHEN1: *mut c_uint = (1056964608 + 2097256) as *mut c_uint;

const HOST_SPEC_NUM: c_uint = 16711680;
const HOST_SPEC_NUM_SHIFT: c_int = 16;
const C1_SRST_HC: c_uint = 16777216;
const C1_CLK_INTLEN: c_int = 1;
const C1_TOUNIT_MAX: c_int = 917504;
const ACMD41_CMD_COMPLETE: c_long = 2147483648;
const ACMD41_ARG_HC: c_uint = 1375698944;
const ACMD41_VOLTAGE: c_long = 16744448;
const ACMD41_CMD_CCS: c_long = 1073741824;
const CMD_CARD_SELECT: c_uint = 117637120;
const CMD_SEND_SCR: c_uint = 857866256 | 2147483648;
const SR_READ_AVAILABLE: c_uint = 2048;
const SCR_SD_BUS_WIDTH_4: c_long = 1024;
const CMD_SET_BUS_WIDTH: c_uint = 100794368 | 2147483648;
const C0_HCTL_DWITDH: c_uint = 2;

impl Registers {
    /// Wait for data or command ready
    pub fn status(&self, mask: u32) -> Result<(), Error> {
        let mut cnt = 500000;
        while (self.STATUS.read() & mask) != 0 && !self.INTERRUPT.read().is_err() && cnt != 0 {
            wait_msec(1);
            cnt -= 1;
        }
        if cnt <= 0 || self.INTERRUPT.read().is_err() {
            Err(Error::Busy)
        } else {
            Ok(())
        }
    }

    /// Wait for interrupt
    pub fn interrupt(&mut self, mut mask: InterruptFlags) -> Result<(), Error>{
        let mut m = mask | InterruptFlags::ERROR_MASK;
        let mut cnt = 1000000;
        while !self.INTERRUPT.read().intersects(m) && cnt != 0 {
            wait_msec(1);
            cnt -= 1;
        }
        let mut r = self.INTERRUPT.read();
        if cnt <= 0 || r.intersects(InterruptFlags::CTO_ERR | InterruptFlags::DTO_ERR) {
            self.INTERRUPT.write(r);
            Err(Error::Timeout)
        } else if r.is_err() {
            self.INTERRUPT.write(r);
            Err(Error::General)
        } else {
            self.INTERRUPT.write(mask);
            Ok(())
        }
    }
}

/// Wait for data or command ready
pub unsafe fn sd_status(mask: u32) -> Result<(), Error> {
    (*EMMC).status(mask)
}

/// Wait for interrupt
pub unsafe fn sd_int(mut mask: c_uint) -> c_int {
    match (*EMMC).interrupt(InterruptFlags::from_bits(mask).unwrap()) {
        Ok(()) => 0,
        Err(Error::Timeout) => SD_TIMEOUT,
        Err(_) => SD_ERROR,
    }
}


/// Send a command
pub unsafe fn sd_cmd(mut code: c_uint, mut arg: c_uint) -> c_int {
    let mut r = 0u64; // make gcc happy
    sd_err = SD_OK as u64;
    if code & CMD_NEED_APP != 0 {
        r =
            sd_cmd((CMD_APP_CMD | if sd_rca != 0 { CMD_RSPNS_48 } else { 0 }) as u32,
                   sd_rca as u32) as u64;
        if sd_rca != 0 && r == 0 {
            println!("ERROR: failed to send SD APP command");
            sd_err = SD_ERROR as u64;
            return 0;
        }
        code &= !CMD_NEED_APP;
    }
    if sd_status(SR_CMD_INHIBIT).is_err() {
        println!("ERROR: EMMC busy");
        sd_err = SD_TIMEOUT as u64;
        return 0;
    }
    println!("EMMC: Sending command {:08x} arg {:08x}", code, arg);
    (*EMMC).INTERRUPT.write((*EMMC).INTERRUPT.read());
    (*EMMC).ARG1.write(arg);
    (*EMMC).CMDTM.write(code);
    if code == CMD_SEND_OP_COND {
        wait_msec(1000);
    } else if code == CMD_SEND_IF_COND || code == CMD_APP_CMD {
        wait_msec(100);
    }
    if { r = sd_int(INT_CMD_DONE) as u64; r } != 0 {
        println!("ERROR: failed to send EMMC command");
        sd_err = r;
        return 0;
    }
    r = (*EMMC).RESP0.read() as u64;
    if code == CMD_GO_IDLE || code == CMD_APP_CMD {
        return 0;
    } else if code == (CMD_APP_CMD | CMD_RSPNS_48) {
        return (r & SR_APP_CMD) as i32;
    } else if code == CMD_SEND_OP_COND {
        return r as i32;
    } else if code == CMD_SEND_IF_COND {
        return if r == arg as u64 { SD_OK } else { SD_ERROR };
    } else if code == CMD_ALL_SEND_CID {
        r |= (*EMMC).RESP3.read() as u64;
        r |= (*EMMC).RESP2.read() as u64;
        r |= (*EMMC).RESP1.read() as u64;
        return r as i32;
    } else if code == CMD_SEND_REL_ADDR {
        sd_err = ((r & 8191) | ((r & 8192) << 6) | ((r & 16384) << 8) |
                 ((r & 32768) << 8)) & CMD_ERRORS_MASK;
        return (r & CMD_RCA_MASK) as i32;
    }
    return (r & CMD_ERRORS_MASK) as i32;
}

/// read a block from sd card and return the number of bytes read
/// returns 0 on error.
pub unsafe fn sd_readblock(mut lba: c_uint, mut buffer: *mut u8, mut num: c_uint) -> c_int {
    let mut r: u64;
    let mut c = 0;

    if num < 1 { num = 1; }

    println!("sd_readblock lba {:x} num {:x}", lba, num);
    if sd_status(SR_DAT_INHIBIT).is_err() { sd_err = SD_TIMEOUT as u64; return 0; }

    let mut buf = buffer as *mut u32;
    if sd_scr[0] & SCR_SUPP_CCS != 0 {
        if num > 1 && (sd_scr[0] & SCR_SUPP_SET_BLKCNT) != 0 {
            sd_cmd(CMD_SET_BLOCKCNT, num);
            if sd_err != 0 { return 0; };
        }
        (*EMMC).BLKSIZECNT.write((num << 16) | 512);
        sd_cmd(if num == 1 { CMD_READ_SINGLE } else { CMD_READ_MULTI }, lba);
        if sd_err != 0 { return 0; };
    } else {
        (*EMMC).BLKSIZECNT.write((1 << 16) | 512);
    }

    while c < num {
        if (sd_scr[0] & SCR_SUPP_CCS) == 0 {
            sd_cmd(CMD_READ_SINGLE, (lba + c) * 512);
            if sd_err != 0 { return 0; };
        }
        if { r = sd_int(INT_READ_RDY) as u64; r } != 0 {
            uart_puts("\rERROR: Timeout waiting for ready to read\n");
            sd_err = r;
            return 0;
        }
        for d in 0..128 {
            *buf.offset(d) = (*EMMC).DATA.read();
        }
        c += 1;
        buf = buf.offset(128);
    }

    if num > 1 && (sd_scr[0] & SCR_SUPP_SET_BLKCNT) == 0 &&
           (sd_scr[0] & SCR_SUPP_CCS) != 0 {
        sd_cmd(CMD_STOP_TRANS, 0);
    }
    return if sd_err != SD_OK as u64 || c != num { 0 } else { num as i32 * 512 };
}

/// set SD clock to frequency in Hz
pub unsafe fn sd_clk(mut f: c_uint) -> c_int {
    let mut d: c_uint;
    let mut c = 41666666 / f;
    let mut s = 32;
    let mut h = 0;
    let mut cnt = 100000;
    while ((*EMMC).STATUS.read() & (SR_CMD_INHIBIT | SR_DAT_INHIBIT)) != 0 &&
              { let mut _t = cnt; cnt -= 1; _t } != 0 {
        wait_msec(1);
    }
    if cnt <= 0 {
        uart_puts("ERROR: timeout waiting for inhibit flag\n");
        return SD_ERROR;
    }
    (*EMMC).CONTROL1.and_mask(!C1_CLK_EN);
    wait_msec(10);
    let mut x = c - 1;
    if x == 0 {
        s = 0;
    } else {
        if (x & 4294901760) == 0 { x <<= 16; s -= 16; }
        if (x & 4278190080) == 0 { x <<= 8; s -= 8; }
        if (x & 4026531840) == 0 { x <<= 4; s -= 4; }
        if (x & 3221225472) == 0 { x <<= 2; s -= 2; }
        if (x & 2147483648) == 0 { x <<= 1; s -= 1; }
        if s > 0 { s -= 1; }
        if s > 7 { s = 7; };
    }
    if sd_hv > HOST_SPEC_V2 { d = c; } else { d = (1 << s); }
    if d <= 2 { d = 2; s = 0; }
    println!("sd_clk divisor {:x}, shift {:x}", d, s);
    if sd_hv > HOST_SPEC_V2 { h = (d & 768) >> 2; }
    d = (((d & 255) << 8) | h);
    (*EMMC).CONTROL1.write(((*EMMC).CONTROL1.read() & 4294901823) | d);
    wait_msec(10);
    (*EMMC).CONTROL1.or_mask(C1_CLK_EN);
    wait_msec(10);
    cnt = 10000;
    while ((*EMMC).CONTROL1.read() & C1_CLK_STABLE) == 0 &&
              { let mut _t = cnt; cnt -= 1; _t } != 0 {
        wait_msec(10);
    }
    if cnt <= 0 {
        uart_puts("ERROR: failed to get stable clock\n");
        return SD_ERROR;
    }
    return SD_OK;
}

/// initialize EMMC to read SDHC card
pub unsafe fn sd_init() -> c_int {
    let mut ccs = 0; // add software flag

    {
        for &pin in &[47, 48, 49, 50, 51, 52, 53] {
            let mut pin = gpio::Gpio::new(pin).into_alt(gpio::Function::Alt3);
            pin.pull(gpio::Pull::Up)
        }
        *GPHEN1 = *GPHEN1 | (1 << 15);
    }

    let mut r: u64;

    sd_hv = (((*EMMC).SLOTISR_VER.read() & HOST_SPEC_NUM) >> HOST_SPEC_NUM_SHIFT) as u64;
    println!("EMMC: GPIO set up");
    (*EMMC).CONTROL0.write(0);
    (*EMMC).CONTROL1.or_mask(C1_SRST_HC);
    let mut cnt = 10000;
    loop {
        wait_msec(10);
        if !(((*EMMC).CONTROL1.read() & C1_SRST_HC) != 0 &&
                 { let t = cnt; cnt -= 1; t } != 0) {
            break
        };
    }
    if cnt <= 0 {
        println!("ERROR: failed to reset EMMC");
        return SD_ERROR;
    }
    println!("EMMC: reset OK");
    (*EMMC).CONTROL1.or_mask((C1_CLK_INTLEN | C1_TOUNIT_MAX) as u32);
    wait_msec(10);
    if { r = sd_clk(400000) as u64; r } != 0 { return r as i32; }
    (*EMMC).INT_EN.write(4294967295);
    (*EMMC).INT_MASK.write(4294967295);
    sd_scr[0] =
        {
            sd_scr[1] = { sd_rca = { sd_err = 0; sd_err }; sd_rca };
            sd_scr[1]
        };
    sd_cmd(CMD_GO_IDLE, 0);
    if sd_err != 0 { return sd_err as i32; }
    sd_cmd(CMD_SEND_IF_COND, 426);
    if sd_err != 0 { return sd_err as i32; }
    cnt = 6;
    r = 0;
    while (r & ACMD41_CMD_COMPLETE) == 0 &&
              { let mut _t = cnt; cnt -= 1; _t } != 0 {
        wait_cycles(400);
        r = sd_cmd(CMD_SEND_OP_COND, ACMD41_ARG_HC) as u64;
        uart_puts("EMMC: CMD_SEND_OP_COND returned ");
        if r & ACMD41_CMD_COMPLETE != 0 { print!("COMPLETE "); }
        if r & ACMD41_VOLTAGE != 0 { print!("VOLTAGE "); }
        if r & ACMD41_CMD_CCS != 0 { print!("CCS "); }
        println!("{:x}", r);
        if sd_err != SD_TIMEOUT as u64 && sd_err != SD_OK as u64 {
            uart_puts("ERROR: EMMC ACMD41 returned error\n");
            return sd_err as i32;
        };
    }
    if (r & ACMD41_CMD_COMPLETE) == 0 || cnt == 0 { return SD_TIMEOUT; }
    if (r & ACMD41_VOLTAGE) == 0 { return SD_ERROR; }
    if r & ACMD41_CMD_CCS != 0 { ccs = SCR_SUPP_CCS; }
    sd_cmd(CMD_ALL_SEND_CID, 0);
    sd_rca = sd_cmd(CMD_SEND_REL_ADDR, 0) as u64;
    println!("EMMC: CMD_SEND_REL_ADDR returned {:x}", sd_rca);
    if sd_err != 0 { return sd_err as i32; }
    if { r = sd_clk(25000000) as u64; r } != 0 { return r as i32; }
    sd_cmd(CMD_CARD_SELECT, sd_rca as u32);
    if sd_err != 0 { return sd_err as i32; }
    if sd_status(SR_DAT_INHIBIT).is_err() { return SD_TIMEOUT as i32; }
    (*EMMC).BLKSIZECNT.write((1 << 16) | 8);
    sd_cmd(CMD_SEND_SCR, 0);
    if sd_err != 0 { return sd_err as i32; }
    if sd_int(INT_READ_RDY) != 0 { return SD_TIMEOUT as i32; }
    r = 0;
    cnt = 100000;
    while r < 2 && cnt != 0 {
        if (*EMMC).STATUS.read() & SR_READ_AVAILABLE != 0 {
            sd_scr[{ let mut _t = r; r += 1; _t } as usize] = (*EMMC).DATA.read() as u64;
        } else { wait_msec(1); };
    }
    if r != 2 { return SD_TIMEOUT; }
    if sd_scr[0] & SCR_SD_BUS_WIDTH_4 != 0 {
        sd_cmd(CMD_SET_BUS_WIDTH, (sd_rca | 2) as u32);
        if sd_err != 0 { return sd_err as i32; }
        (*EMMC).CONTROL0.or_mask(C0_HCTL_DWITDH);
    }
    print!("EMMC: supports ");
    if sd_scr[0] & SCR_SUPP_SET_BLKCNT != 0 { print!("SET_BLKCNT "); }
    if ccs != 0 { print!("CCS "); }
    print!("\n");
    sd_scr[0] &= !SCR_SUPP_CCS;
    sd_scr[0] |= ccs;
    return SD_OK as i32;
}








const IX_GO_IDLE_STATE: usize =     0;
const IX_ALL_SEND_CID: usize =      1;
const IX_SEND_REL_ADDR: usize =     2;
const IX_SET_DSR: usize =           3;
const IX_SWITCH_FUNC: usize =       4;
const IX_CARD_SELECT: usize =       5;
const IX_SEND_IF_COND: usize =      6;
const IX_SEND_CSD: usize =          7;
const IX_SEND_CID: usize =          8;
const IX_VOLTAGE_SWITCH: usize =    9;
const IX_STOP_TRANS: usize =       10;
const IX_SEND_STATUS: usize =      11;
const IX_GO_INACTIVE: usize =      12;
const IX_SET_BLOCKLEN: usize =     13;
const IX_READ_SINGLE: usize =      14;
const IX_READ_MULTI: usize =       15;
const IX_SEND_TUNING: usize =      16;
const IX_SPEED_CLASS: usize =      17;
const IX_SET_BLOCKCNT: usize =     18;
const IX_WRITE_SINGLE: usize =     19;
const IX_WRITE_MULTI: usize =      20;
const IX_PROGRAM_CSD: usize =      21;
const IX_SET_WRITE_PR: usize =     22;
const IX_CLR_WRITE_PR: usize =     23;
const IX_SND_WRITE_PR: usize =     24;
const IX_ERASE_WR_ST: usize =      25;
const IX_ERASE_WR_END: usize =     26;
const IX_ERASE: usize =            27;
const IX_LOCK_UNLOCK: usize =      28;
const IX_APP_CMD: usize =          29;
const IX_APP_CMD_RCA: usize =      30;
const IX_GEN_CMD: usize =          31;

// Commands hereafter require APP_CMD.
const IX_APP_CMD_START: usize =    32;
const IX_SET_BUS_WIDTH: usize =    32;
const IX_SD_STATUS: usize =        33;
const IX_SEND_NUM_WRBL: usize =    34;
const IX_SEND_NUM_ERS: usize =     35;
const IX_APP_SEND_OP_COND: usize = 36;
const IX_SET_CLR_DET: usize =      37;
const IX_SEND_SCR: usize =         38;


struct EMMCCommand {
    name: &'static str,
    code: u32,
    use_rca: bool,
    delay: u16,
}

impl EMMCCommand {
    const fn new_no(index: u32, name: &'static str) -> Self  {
        Self {
            name: name,
            code: (index & 0b111111) << 24 | CMD_NO_RESP,
            use_rca: false,
            delay: 0,
        }
    }

    const fn new136(index: u32, name: &'static str) -> Self  {
        Self {
            name: name,
            code: (index & 0b111111) << 24 | CMD_136BIT_RESP,
            use_rca: false,
            delay: 0,
        }
    }

    const fn new_48(index: u32, name: &'static str) -> Self  {
        Self {
            name: name,
            code: (index & 0b111111) << 24 | CMD_48BIT_RESP,
            use_rca: false,
            delay: 0,
        }
    }

    const fn new_bs(index: u32, name: &'static str) -> Self  {
        Self {
            name: name,
            code: (index & 0b111111) << 24 | CMD_BUSY48BIT_RESP,
            use_rca: false,
            delay: 0,
        }
    }

    const fn use_rca(self) -> Self {
        Self {
            name: self.name,
            code: self.code,
            use_rca: true,
            delay: self.delay,
        }
    }
    const fn delay(self, delay: u16) -> Self {
        Self {
            name: self.name,
            code: self.code,
            use_rca: self.use_rca,
            delay: delay,
        }
    }

    const fn flags(self, flags: u32) -> Self {
        Self {
            name: self.name,
            code: self.code | flags,
            use_rca: self.use_rca,
            delay: self.delay,
        }
    }
}

const CMD_NO_RESP: u32        = 0b00 << 16; // no response
const CMD_136BIT_RESP: u32    = 0b01 << 16; // 136 bits response
const CMD_48BIT_RESP: u32     = 0b10 << 16; // 48 bits response
const CMD_BUSY48BIT_RESP: u32 = 0b11 << 16; // 48 bits response using busy

const CMD_ISDATA: u32 = 1 << 21;

const TM_MULTI_BLOCK: u32 = 1 << 5;
const TM_DAT_DIR: u32 = 1 << 4;
const TM_BLKCNT_EN: u32 = 1 << 1;

static COMMAND_TABLE: [EMMCCommand; 39] = [
    EMMCCommand::new_no(0x00, "GO_IDLE_STATE"),
    EMMCCommand::new136(0x02, "ALL_SEND_CID" ),
    EMMCCommand::new_48(0x03, "SEND_REL_ADDR"),
    EMMCCommand::new_no(0x04, "SET_DSR"      ),
    EMMCCommand::new_48(0x06, "SWITCH_FUNC"  ),
    EMMCCommand::new_bs(0x07, "CARD_SELECT"  ).use_rca(),
    EMMCCommand::new_48(0x08, "SEND_IF_COND" ).delay(100),
    EMMCCommand::new136(0x09, "SEND_CSD"     ).use_rca(),
    EMMCCommand::new136(0x0A, "SEND_CID"     ).use_rca(),
    EMMCCommand::new_48(0x0B, "VOLT_SWITCH"  ),
    EMMCCommand::new_bs(0x0C, "STOP_TRANS"   ),
    EMMCCommand::new_48(0x0D, "SEND_STATUS"  ).use_rca(),
    EMMCCommand::new_no(0x0F, "GO_INACTIVE"  ).use_rca(),
    EMMCCommand::new_48(0x10, "SET_BLOCKLEN" ),

    EMMCCommand::new_48(0x11, "READ_SINGLE"  ).flags(CMD_ISDATA | TM_DAT_DIR),
    EMMCCommand::new_48(0x12, "READ_MULTI"   ).flags(CMD_ISDATA | TM_DAT_DIR | TM_BLKCNT_EN | TM_MULTI_BLOCK),

    EMMCCommand::new_48(0x13, "SEND_TUNING"  ),
    EMMCCommand::new_bs(0x14, "SPEED_CLASS"  ),
    EMMCCommand::new_48(0x17, "SET_BLOCKCNT" ),
    EMMCCommand::new_48(0x18, "WRITE_SINGLE" ).flags(CMD_ISDATA),
    EMMCCommand::new_48(0x19, "WRITE_MULTI"  ).flags(CMD_ISDATA | TM_BLKCNT_EN | TM_MULTI_BLOCK),

    EMMCCommand::new_48(0x1B, "PROGRAM_CSD"  ),
    EMMCCommand::new_bs(0x1C, "SET_WRITE_PR" ),
    EMMCCommand::new_bs(0x1D, "CLR_WRITE_PR" ),
    EMMCCommand::new_48(0x1E, "SND_WRITE_PR" ),
    EMMCCommand::new_48(0x20, "ERASE_WR_ST"  ),
    EMMCCommand::new_48(0x21, "ERASE_WR_END" ),
    EMMCCommand::new_bs(0x26, "ERASE"        ),
    EMMCCommand::new_48(0x2A, "LOCK_UNLOCK"  ),
    EMMCCommand::new_no(0x37, "APP_CMD"      ).delay(100),
    EMMCCommand::new_48(0x37, "APP_CMD"      ).use_rca(),
    EMMCCommand::new_48(0x38, "GEN_CMD"      ),

    // APP commands must be prefixed by an APP_CMD.
    EMMCCommand::new_48(0x06, "SET_BUS_WIDTH"),
    EMMCCommand::new_48(0x0D, "SD_STATUS"    ).use_rca(),
    EMMCCommand::new_48(0x16, "SEND_NUM_WRBL"),
    EMMCCommand::new_48(0x17, "SEND_NUM_ERS" ),
    EMMCCommand::new_48(0x29, "SD_SENDOPCOND").delay(1000),
    EMMCCommand::new_48(0x2A, "SET_CLR_DET"  ),
    EMMCCommand::new_48(0x33, "SEND_SCR"     ).flags(CMD_ISDATA | TM_DAT_DIR),
];
