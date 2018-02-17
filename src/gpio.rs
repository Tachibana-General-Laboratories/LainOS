use std::intrinsics::{volatile_load, volatile_store};
use std::marker::PhantomData;

pub use pi::IO_BASE;

pub const GPFSEL0: Mmio<u32> =   Mmio::new(IO_BASE + 0x00200000);
pub const GPFSEL1: Mmio<u32> =   Mmio::new(IO_BASE + 0x00200004);
pub const GPFSEL2: Mmio<u32> =   Mmio::new(IO_BASE + 0x00200008);
pub const GPFSEL3: Mmio<u32> =   Mmio::new(IO_BASE + 0x0020000C);
pub const GPFSEL4: Mmio<u32> =   Mmio::new(IO_BASE + 0x00200010);
pub const GPFSEL5: Mmio<u32> =   Mmio::new(IO_BASE + 0x00200014);
pub const GPSET0: Mmio<u32> =    Mmio::new(IO_BASE + 0x0020001C);
pub const GPSET1: Mmio<u32> =    Mmio::new(IO_BASE + 0x00200020);
pub const GPCLR0: Mmio<u32> =    Mmio::new(IO_BASE + 0x00200028);
pub const GPLEV0: Mmio<u32> =    Mmio::new(IO_BASE + 0x00200034);
pub const GPLEV1: Mmio<u32> =    Mmio::new(IO_BASE + 0x00200038);
pub const GPEDS0: Mmio<u32> =    Mmio::new(IO_BASE + 0x00200040);
pub const GPEDS1: Mmio<u32> =    Mmio::new(IO_BASE + 0x00200044);
pub const GPHEN0: Mmio<u32> =    Mmio::new(IO_BASE + 0x00200064);
pub const GPHEN1: Mmio<u32> =    Mmio::new(IO_BASE + 0x00200068);
pub const GPPUD: Mmio<u32> =     Mmio::new(IO_BASE + 0x00200094);
pub const GPPUDCLK0: Mmio<u32> = Mmio::new(IO_BASE + 0x00200098);
pub const GPPUDCLK1: Mmio<u32> = Mmio::new(IO_BASE + 0x0020009C);

/// Generates `pub enums` with no variants for each `ident` passed in.
pub macro states($($name:ident),*) {
    $(pub enum $name {  })*
}

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
