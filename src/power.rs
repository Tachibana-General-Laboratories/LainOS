use gpio::*;
use mbox;
use delays;

const PM_RSTC: Mmio<u32> = Mmio::new(MMIO_BASE + 0x0010001c);
const PM_RSTS: Mmio<u32> = Mmio::new(MMIO_BASE + 0x00100020);
const PM_WDOG: Mmio<u32> = Mmio::new(MMIO_BASE + 0x00100024);

const PM_WDOG_MAGIC: u32   = 0x5a000000;
const PM_RSTC_FULLRST: u32 = 0x00000020;

/// Shutdown the board
pub fn power_off() {
    let b = unsafe { &mut mbox::BUFFER };

    // power off devices one by one
    for device_id in 0..16 {
        b[0].write(8*4);
        b[1].write(mbox::REQUEST);
        b[2].write(mbox::TAG_SETPOWER); // set power state
        b[3].write(8);
        b[4].write(8);
        b[5].write(device_id);   // device id
        b[6].write(0);           // bit 0: off, bit 1: no wait
        b[7].write(mbox::TAG_LAST);
        unsafe {
            mbox::call(mbox::Channel::PROP1);
        }
    }

    // power off gpio pins (but not VCC pins)
    GPFSEL0.write(0);
    GPFSEL1.write(0);
    GPFSEL2.write(0);
    GPFSEL3.write(0);
    GPFSEL4.write(0);
    GPFSEL5.write(0);
    GPPUD.write(0);
    delays::wait_cycles(150);

    GPPUDCLK0.write(0xffffffff);
    GPPUDCLK1.write(0xffffffff);
    delays::wait_cycles(150);

    // flush GPIO setup
    GPPUDCLK0.write(0);
    GPPUDCLK1.write(0);

    // power off the SoC (GPU + CPU)
    let mut r = PM_RSTS.read();
    r &= !0xfffffaaa;
    r |= 0x555;    // partition 63 used to indicate halt
    PM_RSTS.write(PM_WDOG_MAGIC | r);
    PM_WDOG.write(PM_WDOG_MAGIC | 10);
    PM_RSTC.write(PM_WDOG_MAGIC | PM_RSTC_FULLRST);
}

/// Reboot
pub fn reset() {
    // trigger a restart by instructing the GPU to boot from partition 0
    let mut r = PM_RSTS.read();
    r &= !0xfffffaaa;
    PM_RSTS.write(PM_WDOG_MAGIC | r);   // boot from partition 0
    PM_WDOG.write(PM_WDOG_MAGIC | 10);
    PM_RSTC.write(PM_WDOG_MAGIC | PM_RSTC_FULLRST);
}
