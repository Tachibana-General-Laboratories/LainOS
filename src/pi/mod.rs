/// The address where I/O peripherals are mapped to.
pub const IO_BASE: usize = 0x3F000000;

/// Generates `pub enums` with no variants for each `ident` passed in.
pub macro states($($name:ident),*) {
    $(pub enum $name {  })*
}


pub mod gpio;
pub mod timer;
pub mod mbox;
pub mod power;
pub mod uart0;

static mut GP: *mut gpio::Registers = gpio::GPIO_BASE as *mut gpio::Registers;
