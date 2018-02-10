use core::intrinsics::{volatile_load, volatile_store};
use core::mem::uninitialized;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::marker::PhantomData;

pub const MMIO_BASE: usize = 0x3F000000;

pub const GPFSEL0: Mmio<u32> =   Mmio::new(MMIO_BASE + 0x00200000);
pub const GPFSEL1: Mmio<u32> =   Mmio::new(MMIO_BASE + 0x00200004);
pub const GPFSEL2: Mmio<u32> =   Mmio::new(MMIO_BASE + 0x00200008);
pub const GPFSEL3: Mmio<u32> =   Mmio::new(MMIO_BASE + 0x0020000C);
pub const GPFSEL4: Mmio<u32> =   Mmio::new(MMIO_BASE + 0x00200010);
pub const GPFSEL5: Mmio<u32> =   Mmio::new(MMIO_BASE + 0x00200014);
pub const GPSET0: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x0020001C);
pub const GPSET1: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x00200020);
pub const GPCLR0: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x00200028);
pub const GPLEV0: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x00200034);
pub const GPLEV1: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x00200038);
pub const GPEDS0: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x00200040);
pub const GPEDS1: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x00200044);
pub const GPHEN0: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x00200064);
pub const GPHEN1: Mmio<u32> =    Mmio::new(MMIO_BASE + 0x00200068);
pub const GPPUD: Mmio<u32> =     Mmio::new(MMIO_BASE + 0x00200094);
pub const GPPUDCLK0: Mmio<u32> = Mmio::new(MMIO_BASE + 0x00200098);
pub const GPPUDCLK1: Mmio<u32> = Mmio::new(MMIO_BASE + 0x0020009C);

pub struct Mmio<T> {
    addr: usize,
    value: PhantomData<T>,
}

impl<T> Mmio<T> {
    pub const fn new(addr: usize) -> Self {
        Self {
            addr,
            value: PhantomData,
        }
    }

    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { volatile_load(self.addr as *const T) }
    }

    #[inline(always)]
    pub fn write(&mut self, value: T) {
        unsafe { volatile_store(self.addr as *mut T, value) }
    }
}

/*
impl<T: BitAnd<Output=T>> BitAndAssign<T> for Mmio<T> {
    fn bitand_assign(&mut self, rhs: T) {
        let v = self.read();
        self.write(v & rhs);
    }
}

impl<T: BitOr<Output=T>> BitOrAssign<T> for Mmio<T> {
    fn bitor_assign(&mut self, rhs: T) {
        let v = self.read();
        self.write(v | rhs);
    }
}
*/

//, BitOr, Not

/*
//use super::io::Io;

#[repr(packed)]
pub struct MMIO {
    addr: usize
    value: T,
}


    fn read(&self) -> T {
        unsafe { volatile_load(&self.value) }
    }

    fn write(&mut self, value: T) {
        unsafe { volatile_store(&mut self.value, value) };
    }
}

/*
impl<T> Io for Mmio<T> where T: Copy + PartialEq + BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T> {
    type Value = T;

}
*/
*/
