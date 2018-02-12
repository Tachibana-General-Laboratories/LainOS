use gpio::*;
use util;

use volatile::prelude::*;
use volatile::{Volatile, Reserved};

struct Registers {
    ARG2: Volatile<u32>, // ACMD23 Argument
    BLKSIZECNT: Volatile<u32>, // Block Size and Count
    ARG1: Volatile<u32>, // Argument 
    RESP0: Volatile<u32>, // Response bits  31 : 0
    RESP1: Volatile<u32>, // Response bits  63 : 32
    RESP2: Volatile<u32>, // Response bits  95 : 64
    RESP3: Volatile<u32>, // Response bits 127 : 96
    DATA: Volatile<u32>, // Data
    STATUS: Volatile<u32>, // Status
    CONTROL0: Volatile<u32>, // Host Configuration bits 
    CONTROL1: Volatile<u32>, // Host Configuration bits 
    INTERRUPT: Volatile<u32>, // Interrupt Flags 
    IRPT_MASK: Volatile<u32>, // Interrupt Flag Enable 
    IRPT_EN: Volatile<u32>, // Interrupt Generation Enable 
    CONTROL2: Volatile<u32>, // Host Configuration bits 
    FORCE_IRPT: Volatile<u32>, // Force Interrupt Event 
    BOOT_TIMEOUT: Volatile<u32>, // Timeout in boot mode 
    DBG_SEL: Volatile<u32>, // Debug Bus Configuration 
    EXRDFIFO_CFG: Volatile<u32>, // Extension FIFO Configuration 
    EXRDFIFO_EN: Volatile<u32>, // Extension FIFO Enable 
    TUNE_STEP: Volatile<u32>, // Delay per card clock tuning step 
    TUNE_STEPS_STD: Volatile<u32>, // Card clock tuning steps for SDR 
    TUNE_STEPS_DDR: Volatile<u32>, // Card clock tuning steps for DDR 
}

/// SPI Interrupt Support
const SPI_INT_SPT: *mut Volatile<u32> = (IO_BASE + 0x3000f0) as *mut Volatile<u32>;

/// Slot Interrupt Status and Version
const SLOTISR_VER: *mut Volatile<u32> = (IO_BASE + 0x3000fc) as *mut Volatile<u32>;



//*

pub const OK: u32 = 0;
pub const TIMEOUT: u32 = 0xFFFF_FFFF - 1;
pub const ERROR: u32 = 0xFFFF_FFFF - 2;

const EMMC_ARG2: Mmio<u32>         = Mmio::new(IO_BASE + 0x00300000);
const EMMC_BLKSIZECNT: Mmio<u32>   = Mmio::new(IO_BASE + 0x00300004);
const EMMC_ARG1: Mmio<u32>         = Mmio::new(IO_BASE + 0x00300008);
const EMMC_CMDTM: Mmio<u32>        = Mmio::new(IO_BASE + 0x0030000C);
const EMMC_RESP0: Mmio<u32>        = Mmio::new(IO_BASE + 0x00300010);
const EMMC_RESP1: Mmio<u32>        = Mmio::new(IO_BASE + 0x00300014);
const EMMC_RESP2: Mmio<u32>        = Mmio::new(IO_BASE + 0x00300018);
const EMMC_RESP3: Mmio<u32>        = Mmio::new(IO_BASE + 0x0030001C);
const EMMC_DATA: Mmio<u32>         = Mmio::new(IO_BASE + 0x00300020);
const EMMC_STATUS: Mmio<u32>       = Mmio::new(IO_BASE + 0x00300024);
const EMMC_CONTROL0: Mmio<u32>     = Mmio::new(IO_BASE + 0x00300028);
const EMMC_CONTROL1: Mmio<u32>     = Mmio::new(IO_BASE + 0x0030002C);
const EMMC_INTERRUPT: Mmio<u32>    = Mmio::new(IO_BASE + 0x00300030);
const EMMC_INT_MASK: Mmio<u32>     = Mmio::new(IO_BASE + 0x00300034);
const EMMC_INT_EN: Mmio<u32>       = Mmio::new(IO_BASE + 0x00300038);
const EMMC_CONTROL2: Mmio<u32>     = Mmio::new(IO_BASE + 0x0030003C);
const EMMC_SLOTISR_VER: Mmio<u32>  = Mmio::new(IO_BASE + 0x003000FC);

// command flags
const CMD_NEED_APP: u32 =        0x80000000;
const CMD_RSPNS_48: u32 =        0x00020000;
const CMD_ERRORS_MASK: u32 =     0xfff9c004;
const CMD_RCA_MASK: u32 =        0xffff0000;

// COMMANDs
const CMD_GO_IDLE: u32 =         0x00000000;
const CMD_ALL_SEND_CID: u32 =    0x02010000;
const CMD_SEND_REL_ADDR: u32 =   0x03020000;
const CMD_CARD_SELECT: u32 =     0x07030000;
const CMD_SEND_IF_COND: u32 =    0x08020000;
const CMD_STOP_TRANS: u32 =      0x0C030000;
const CMD_READ_SINGLE: u32 =     0x11220010;
const CMD_READ_MULTI: u32 =      0x12220032;
const CMD_SET_BLOCKCNT: u32 =    0x17020000;
const CMD_APP_CMD: u32 =         0x37000000;
const CMD_SET_BUS_WIDTH: u32 =   0x06020000 | CMD_NEED_APP;
const CMD_SEND_OP_COND: u32 =    0x29020000 | CMD_NEED_APP;
const CMD_SEND_SCR: u32 =        0x33220010 | CMD_NEED_APP;

// STATUS register settings
const SR_READ_AVAILABLE: u32 =   0x00000800;
const SR_DAT_INHIBIT: u32 =      0x00000002;
const SR_CMD_INHIBIT: u32 =      0x00000001;
const SR_APP_CMD: u32 =          0x00000020;

// INTERRUPT register settings
const INT_DATA_TIMEOUT: u32 =    0x00100000;
const INT_CMD_TIMEOUT: u32 =     0x00010000;
const INT_READ_RDY: u32 =        0x00000020;
const INT_CMD_DONE: u32 =        0x00000001;

const INT_ERROR_MASK: u32 =      0x017E8000;

// CONTROL register settings
const C0_SPI_MODE_EN: u32 =      0x00100000;
const C0_HCTL_HS_EN: u32 =       0x00000004;
const C0_HCTL_DWITDH: u32 =      0x00000002;

const C1_SRST_DATA: u32 =        0x04000000;
const C1_SRST_CMD: u32 =         0x02000000;
const C1_SRST_HC: u32 =          0x01000000;
const C1_TOUNIT_DIS: u32 =       0x000f0000;
const C1_TOUNIT_MAX: u32 =       0x000e0000;
const C1_CLK_GENSEL: u32 =       0x00000020;
const C1_CLK_EN: u32 =           0x00000004;
const C1_CLK_STABLE: u32 =       0x00000002;
const C1_CLK_INTLEN: u32 =       0x00000001;

// SLOTISR_VER values
const HOST_SPEC_NUM: u32 =       0x00ff0000;
const HOST_SPEC_NUM_SHIFT: u32 = 16;
const HOST_SPEC_V3: u32 =        2;
const HOST_SPEC_V2: u32 =        1;
const HOST_SPEC_V1: u32 =        0;

// SCR flags
const SCR_SD_BUS_WIDTH_4: u32 =  0x00000400;
const SCR_SUPP_SET_BLKCNT: u32 = 0x02000000;
// added by my driver
const SCR_SUPP_CCS: u32 =        0x00000001;

const ACMD41_VOLTAGE: u32 =      0x00ff8000;
const ACMD41_CMD_COMPLETE: u32 = 0x80000000;
const ACMD41_CMD_CCS: u32 =      0x40000000;
const ACMD41_ARG_HC: u32 =       0x51ff8000;

//unsigned long sd_scr[2], sd_ocr, sd_rca, sd_err, sd_hv;
static mut sd_scr: [u32; 2] = [0; 2];
static mut sd_ocr: u32 = 0;
static mut sd_rca: u32 = 0;
static mut sd_err: u32 = 0;
static mut sd_hv: u32 = 0;

fn cnt_wait<F>(msec: u32, cnt: u32, mut f: F) -> u32
    where F: FnMut() -> bool
{
    let mut cnt = cnt + 1;
    while {
        cnt -= 1;
        f() && cnt != 0
    } {
        util::wait_msec(msec)
    }
    cnt
}

/// Wait for data or command ready
unsafe fn sd_status(mask: u32) -> u32 {
    let cnt = cnt_wait(1, 500000, || (EMMC_STATUS.read() & mask) != 0 && (EMMC_INTERRUPT.read() & INT_ERROR_MASK) == 0);
    if cnt <= 0 || (EMMC_INTERRUPT.read() & INT_ERROR_MASK) != 0 {
        ERROR
    } else {
        OK
    }
}

/// Wait for interrupt
unsafe fn sd_int(mask: u32) -> u32  {
    let m = mask | INT_ERROR_MASK;
    let cnt = cnt_wait(1, 1000000, || (EMMC_INTERRUPT.read() & m) == 0);

    let r = EMMC_INTERRUPT.read();
    if cnt <= 0 || (r & INT_CMD_TIMEOUT) != 0 || (r & INT_DATA_TIMEOUT) != 0 {
        EMMC_INTERRUPT.write(r);
        TIMEOUT
    } else if (r & INT_ERROR_MASK) != 0 {
        EMMC_INTERRUPT.write(r);
        ERROR
    } else {
        EMMC_INTERRUPT.write(mask);
        OK
    }
}

/// Send a command
unsafe fn sd_cmd(mut code: u32, arg: u32) -> u32 {
    //int r=0;
    let mut r = 0;
    sd_err = OK;

    if code&CMD_NEED_APP != 0 {
        let cmd = if sd_rca != 0 { CMD_RSPNS_48 } else { 0 };
        r = sd_cmd(CMD_APP_CMD | cmd, sd_rca);
        if sd_rca != 0 && r == 0 {
            println!("ERROR: failed to send SD APP command");
            sd_err = ERROR;
            return 0;
        }
        code &= !CMD_NEED_APP;
    }

    if sd_status(SR_CMD_INHIBIT) != 0 {
        println!("ERROR: EMMC busy");
        sd_err = TIMEOUT;
        return 0;
    }

    println!("EMMC: Sending command {:X} arg {:X}", code, arg);

    EMMC_INTERRUPT.write(EMMC_INTERRUPT.read());
    EMMC_ARG1.write(arg);
    EMMC_CMDTM.write(code);

    if code == CMD_SEND_OP_COND {
        util::wait_msec(1000);
    } else if code == CMD_SEND_IF_COND || code == CMD_APP_CMD {
        util::wait_msec(100);
    }

    r = sd_int(INT_CMD_DONE);
    if r != 0 {
        println!("ERROR: failed to send EMMC command\n");
        sd_err = r;
        return 0;
    }

    r = EMMC_RESP0.read();

    const _CODE: u32 = CMD_APP_CMD | CMD_RSPNS_48;
    match code {
        CMD_GO_IDLE | CMD_APP_CMD => 0,
        _CODE => r & SR_APP_CMD,
        CMD_SEND_OP_COND => r,
        CMD_SEND_IF_COND => if r == arg { OK } else { ERROR },
        CMD_ALL_SEND_CID => {
            r |= EMMC_RESP3.read();
            r |= EMMC_RESP2.read();
            r |= EMMC_RESP1.read();
            r
        }
        CMD_SEND_REL_ADDR => {
            sd_err = (
                    ((r & 0x1fff)   )|
                    ((r & 0x2000)<<6)|
                    ((r & 0x4000)<<8)|
                    ((r & 0x8000)<<8)
                )&CMD_ERRORS_MASK;
            r & CMD_RCA_MASK
        }
        _ => r & CMD_ERRORS_MASK,
    }
}

// read a block from sd card and return the number of bytes read
// returns 0 on error.
unsafe fn sd_readblock(lba: u32, buffer: &mut [u32]) -> u32 {
    println!("sd_readblock lba {} len {}", lba, buffer.len());

    if sd_status(SR_DAT_INHIBIT) != OK {
        sd_err = TIMEOUT;
        return 0;
    }

    let num = buffer.len() as u32;
    //let buf = buffer.as_mut_ptr();
    let buf = buffer;
    if sd_scr[0] & SCR_SUPP_CCS != 0 {
        if num > 1 && (sd_scr[0] & SCR_SUPP_SET_BLKCNT) != 0  {
            sd_cmd(CMD_SET_BLOCKCNT, num);
            if sd_err != OK {
                return 0;
            }
        }
        EMMC_BLKSIZECNT.write((num << 16) | 512);
        if num == 1 {
            sd_cmd(CMD_READ_SINGLE, lba);
        } else {
            sd_cmd(CMD_READ_MULTI, lba);
        }
        if sd_err != OK {
            return 0;
        }
    } else {
        EMMC_BLKSIZECNT.write((1 << 16) | 512);
    }

    for c in 0..num as usize {
        if !(sd_scr[0] & SCR_SUPP_CCS) != 0 {
            sd_cmd(CMD_READ_SINGLE, (lba + c as u32) * 512);
            if sd_err != OK {
                return 0;
            }
        }
        let r = sd_int(INT_READ_RDY);
        if r != 0 {
            println!("ERROR: Timeout waiting for ready to read");
            sd_err = r;
            return 0;
        }
        for d in 0..128 {
            buf[c * 128 + d] = EMMC_DATA.read();
        }
    }

    if num > 1 && sd_scr[0] & SCR_SUPP_SET_BLKCNT == 0 && sd_scr[0] & SCR_SUPP_CCS != 0 {
        sd_cmd(CMD_STOP_TRANS, 0);
    }

    if sd_err != OK {
        0
    } else {
        num * 512
    }

    //return sd_err != SD_OK || c != num ? 0 : num*512;
}

/// set SD clock to frequency in Hz
unsafe fn sd_clk(f: u32) -> u32 {
    let mut d;
    let mut c = 41666666/f;
    let mut x;
    let mut s = 32;
    let mut h = 0;

    let cnt = cnt_wait(1, 100000, || EMMC_STATUS.read() & (SR_CMD_INHIBIT|SR_DAT_INHIBIT) != 0);

    if cnt <= 0 {
        println!("ERROR: timeout waiting for inhibit flag");
        return ERROR;
    }

    EMMC_CONTROL1.write(EMMC_CONTROL1.read() & !C1_CLK_EN);

    util::wait_msec(10);
    x = c-1;

    if x == 0 { s=0; } else {
        if !(x & 0xffff0000) != 0 { x <<= 16; s -= 16; }
        if !(x & 0xff000000) != 0 { x <<= 8;  s -= 8; }
        if !(x & 0xf0000000) != 0 { x <<= 4;  s -= 4; }
        if !(x & 0xc0000000) != 0 { x <<= 2;  s -= 2; }
        if !(x & 0x80000000) != 0 { x <<= 1;  s -= 1; }
        if s>0 { s -= 1; }
        if s>7 { s = 7; }
    }

    if sd_hv > HOST_SPEC_V2 {
        d=c;
    } else {
        d=(1<<s);
    }
    if d<=2 {
        d=2;
        s=0;
    }

    println!("sd_clk divisor {}, shift {}", d, s);
    if sd_hv > HOST_SPEC_V2 {
        h = (d&0x300) >> 2;
    }
    d = ((d & 0x0ff) << 8) | h;

    EMMC_CONTROL1.write((EMMC_CONTROL1.read() & 0xffff003f) | d);
    util::wait_msec(10);
    EMMC_CONTROL1.write(EMMC_CONTROL1.read() | C1_CLK_EN);
    util::wait_msec(10);

    let cnt = cnt_wait(10, 10000, || (EMMC_CONTROL1.read() & C1_CLK_STABLE) == 0);
    if cnt == 0 {
        println!("ERROR: failed to get stable clock");
        ERROR
    } else {
        OK
    }
}

/// initialize EMMC to read SDHC card
unsafe fn sd_init() -> u32 {
    let mut r;
    let mut ccs;
    // long r,cnt,ccs=0;

    /*
    // GPIO_CD
    r=*GPFSEL4;
    r&=~(7<<(7*3)); *GPFSEL4=r;
    *GPPUD=2;
    wait_cycles(150);
    *GPPUDCLK1=(1<<15);
    wait_cycles(150);
    *GPPUD=0;
    *GPPUDCLK1=0;
    r=*GPHEN1;
    r|=1<<15;
    *GPHEN1=r;

    // GPIO_CLK, GPIO_CMD
    r=*GPFSEL4; r|=(7<<(8*3))|(7<<(9*3)); *GPFSEL4=r;
    *GPPUD=2; wait_cycles(150); *GPPUDCLK1=(1<<16)|(1<<17); wait_cycles(150); *GPPUD=0; *GPPUDCLK1=0;

    // GPIO_DAT0, GPIO_DAT1, GPIO_DAT2, GPIO_DAT3
    r=*GPFSEL5; r|=(7<<(0*3)) | (7<<(1*3)) | (7<<(2*3)) | (7<<(3*3)); *GPFSEL5=r;
    *GPPUD=2; wait_cycles(150);
    *GPPUDCLK1=(1<<18) | (1<<19) | (1<<20) | (1<<21);
    wait_cycles(150); *GPPUD=0; *GPPUDCLK1=0;

    sd_hv = (*EMMC_SLOTISR_VER & HOST_SPEC_NUM) >> HOST_SPEC_NUM_SHIFT;
    uart_puts("EMMC: GPIO set up\n");
    // Reset the card.
    *EMMC_CONTROL0 = 0; *EMMC_CONTROL1 |= C1_SRST_HC;
    cnt=10000; do{wait_msec(10);} while( (*EMMC_CONTROL1 & C1_SRST_HC) && cnt-- );
    if(cnt<=0) {
        uart_puts("ERROR: failed to reset EMMC\n");
        return SD_ERROR;
    }
    uart_puts("EMMC: reset OK\n");
    *EMMC_CONTROL1 |= C1_CLK_INTLEN | C1_TOUNIT_MAX;
    wait_msec(10);
    // Set clock to setup frequency.
    if((r=sd_clk(400000))) return r;
    *EMMC_INT_EN   = 0xffffffff;
    *EMMC_INT_MASK = 0xffffffff;
    sd_scr[0]=sd_scr[1]=sd_rca=sd_err=0;
    sd_cmd(CMD_GO_IDLE,0);
    if(sd_err) return sd_err;

    sd_cmd(CMD_SEND_IF_COND,0x000001AA);
    if(sd_err) return sd_err;
    cnt=6; r=0; while(!(r&ACMD41_CMD_COMPLETE) && cnt--) {
        wait_cycles(400);
        r=sd_cmd(CMD_SEND_OP_COND,ACMD41_ARG_HC);
        uart_puts("EMMC: CMD_SEND_OP_COND returned ");
        if(r&ACMD41_CMD_COMPLETE)
            uart_puts("COMPLETE ");
        if(r&ACMD41_VOLTAGE)
            uart_puts("VOLTAGE ");
        if(r&ACMD41_CMD_CCS)
            uart_puts("CCS ");
        uart_hex(r>>32);
        uart_hex(r);
        uart_send('\n');
        if(sd_err != SD_TIMEOUT && sd_err!=SD_OK ) {
            uart_puts("ERROR: EMMC ACMD41 returned error\n");
            return sd_err;
        }
    }
*/

    if r & ACMD41_CMD_COMPLETE == 0 || cnt == 0 { return TIMEOUT; }
    if r & ACMD41_VOLTAGE == 0 { return SD_ERROR; }
    if r & ACMD41_CMD_CCS != 0 { ccs = SCR_SUPP_CCS;

    sd_cmd(CMD_ALL_SEND_CID, 0);

    sd_rca = sd_cmd(CMD_SEND_REL_ADDR, 0);
    println!("EMMC: CMD_SEND_REL_ADDR returned {}", sd_rca);

    if sd_err != 0 { return sd_err; }
    r = sd_clk(25000000);
    if r != 0 { return r; }

    sd_cmd(CMD_CARD_SELECT, sd_rca);
    if sd_err != 0 { return sd_err; }

    if sd_status(SR_DAT_INHIBIT) != 0 {
        return TIMEOUT;
    }
    EMMC_BLKSIZECNT.write((1<<16) | 8);
    sd_cmd(CMD_SEND_SCR, 0);
    if sd_err != 0 { return sd_err; }
    if sd_int(INT_READ_RDY) != 0 { return TIMEOUT; }

    r = 0;
    let mut cnt = 100000;
    while r < 2 && cnt != 0 {
        if EMMC_STATUS.read() & SR_READ_AVAILABLE != 0 {
            sd_scr[r] = EMMC_DATA.read();
            r += 1;
        } else {
            wait_msec(1);
        }
        cnt -= 1; // XXX: MAYBE?
    }
    if r != 2 {
        return TIMEOUT;
    }

    if sd_scr[0] & SCR_SD_BUS_WIDTH_4 != 0 {
        sd_cmd(CMD_SET_BUS_WIDTH, sd_rca | 2);
        if sd_err { return sd_err; }
        EMMC_CONTROL0.write(EMMC_CONTROL0.read() | C0_HCTL_DWITDH);
    }

    // add software flag
    print!("EMMC: supports ");
    if sd_scr[0] & SCR_SUPP_SET_BLKCNT != 0 {
        print!("SET_BLKCNT ");
    }
    if ccs {
        print("CCS ");
    }
    println!("");
    sd_scr[0] &= !SCR_SUPP_CCS;
    sd_scr[0] |= ccs;
    OK
}
// */
