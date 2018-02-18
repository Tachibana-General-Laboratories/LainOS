use super::*;
use pi::mbox;
use util;

use volatile::prelude::*;
use volatile::Volatile;

const PM_RSTC: *mut Volatile<u32> = (IO_BASE + 0x0010001c) as *mut Volatile<u32>;
const PM_RSTS: *mut Volatile<u32> = (IO_BASE + 0x00100020) as *mut Volatile<u32>;
const PM_WDOG: *mut Volatile<u32> = (IO_BASE + 0x00100024) as *mut Volatile<u32>;

const PM_WDOG_MAGIC: u32   = 0x5a000000;
const PM_RSTC_FULLRST: u32 = 0x00000020;

/// Shutdown the board
pub fn power_off() {
    // power off devices one by one
    let mut b = mbox::Mailbox::new();
    for device_id in 0..16 {
        b[0].write(8*4);
        b[1].write(mbox::REQUEST);
        b[2].write(mbox::TAG_SETPOWER); // set power state
        b[3].write(8);
        b[4].write(8);
        b[5].write(device_id);   // device id
        b[6].write(0);           // bit 0: off, bit 1: no wait
        b[7].write(mbox::TAG_LAST);
        b.call(mbox::Channel::PROP1).unwrap();
    }

    unsafe {
    // power off gpio pins (but not VCC pins)
    (*GPFSEL0).write(0);
    (*GPFSEL1).write(0);
    (*GPFSEL2).write(0);
    (*GPFSEL3).write(0);
    (*GPFSEL4).write(0);
    (*GPFSEL5).write(0);
    (*GPPUD).write(0);
    util::wait_cycles(150);

    (*GPPUDCLK0).write(0xffffffff);
    (*GPPUDCLK1).write(0xffffffff);
    util::wait_cycles(150);

    // flush GPIO setup
    (*GPPUDCLK0).write(0);
    (*GPPUDCLK1).write(0);

    // power off the SoC (GPU + CPU)
    let mut r = (*PM_RSTS).read();
    r &= !0xfffffaaa;
    r |= 0x555;    // partition 63 used to indicate halt
    (*PM_RSTS).write(PM_WDOG_MAGIC | r);
    (*PM_WDOG).write(PM_WDOG_MAGIC | 10);
    (*PM_RSTC).write(PM_WDOG_MAGIC | PM_RSTC_FULLRST);
    }
}

/// Reboot
pub fn reset() {
    unsafe {
    // trigger a restart by instructing the GPU to boot from partition 0
    let mut r = (*PM_RSTS).read();
    r &= !0xfffffaaa;
    (*PM_RSTS).write(PM_WDOG_MAGIC | r);   // boot from partition 0
    (*PM_WDOG).write(PM_WDOG_MAGIC | 10);
    (*PM_RSTC).write(PM_WDOG_MAGIC | PM_RSTC_FULLRST);
    }
}


pub const GPFSEL0: *mut Volatile<u32> =   (IO_BASE + 0x00200000) as *mut Volatile<u32>;
pub const GPFSEL1: *mut Volatile<u32> =   (IO_BASE + 0x00200004) as *mut Volatile<u32>;
pub const GPFSEL2: *mut Volatile<u32> =   (IO_BASE + 0x00200008) as *mut Volatile<u32>;
pub const GPFSEL3: *mut Volatile<u32> =   (IO_BASE + 0x0020000C) as *mut Volatile<u32>;
pub const GPFSEL4: *mut Volatile<u32> =   (IO_BASE + 0x00200010) as *mut Volatile<u32>;
pub const GPFSEL5: *mut Volatile<u32> =   (IO_BASE + 0x00200014) as *mut Volatile<u32>;
pub const GPSET0: *mut Volatile<u32> =    (IO_BASE + 0x0020001C) as *mut Volatile<u32>;
pub const GPSET1: *mut Volatile<u32> =    (IO_BASE + 0x00200020) as *mut Volatile<u32>;
pub const GPCLR0: *mut Volatile<u32> =    (IO_BASE + 0x00200028) as *mut Volatile<u32>;
pub const GPLEV0: *mut Volatile<u32> =    (IO_BASE + 0x00200034) as *mut Volatile<u32>;
pub const GPLEV1: *mut Volatile<u32> =    (IO_BASE + 0x00200038) as *mut Volatile<u32>;
pub const GPEDS0: *mut Volatile<u32> =    (IO_BASE + 0x00200040) as *mut Volatile<u32>;
pub const GPEDS1: *mut Volatile<u32> =    (IO_BASE + 0x00200044) as *mut Volatile<u32>;
pub const GPHEN0: *mut Volatile<u32> =    (IO_BASE + 0x00200064) as *mut Volatile<u32>;
pub const GPHEN1: *mut Volatile<u32> =    (IO_BASE + 0x00200068) as *mut Volatile<u32>;
pub const GPPUD: *mut Volatile<u32> =     (IO_BASE + 0x00200094) as *mut Volatile<u32>;
pub const GPPUDCLK0: *mut Volatile<u32> = (IO_BASE + 0x00200098) as *mut Volatile<u32>;
pub const GPPUDCLK1: *mut Volatile<u32> = (IO_BASE + 0x0020009C) as *mut Volatile<u32>;
