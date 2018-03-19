use common::IO_BASE;
use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile};

const INT_BASE: usize = IO_BASE + 0xB000 + 0x200;

#[derive(Copy, Clone, PartialEq)]
pub enum Interrupt {
    Timer1 = 1,
    Timer3 = 3,
    Usb = 9,
    Gpio0 = 49,
    Gpio1 = 50,
    Gpio2 = 51,
    Gpio3 = 52,
    Uart = 57,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IRQ_basic_pending: ReadVolatile<u32>,
    IRQ_pending_1: ReadVolatile<u32>,
    IRQ_pending_2: ReadVolatile<u32>,
    FIQ_control: Volatile<u32>,
    Enable_IRQs_1: Volatile<u32>,
    Enable_IRQs_2: Volatile<u32>,
    Enable_Basic_IRQs: Volatile<u32>,
    Disable_IRQs_1: Volatile<u32>,
    Disable_IRQs_2: Volatile<u32>,
    Disable_Basic_IRQs: Volatile<u32>,
}

/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct Controller {
    registers: &'static mut Registers
}

impl Controller {
    /// Returns a new handle to the interrupt controller.
    pub fn new() -> Self {
        let registers = unsafe { &mut *(INT_BASE as *mut Registers) };
        Self { registers }
    }

    /// Enables the interrupt `int`.
    pub fn enable(&mut self, int: Interrupt) {
        let int = int as u32;
        if int < 32 {
            self.registers.Enable_IRQs_1.write(1 << int);
        } else {
            self.registers.Enable_IRQs_2.write(1 << (int - 32));
        }
    }

    /// Disables the interrupt `int`.
    pub fn disable(&mut self, int: Interrupt) {
        let int = int as u32;
        if int < 32 {
            self.registers.Disable_IRQs_1.write(1 << int);
        } else {
            self.registers.Disable_IRQs_2.write(1 << (int - 32));
        }
    }

    /// Returns `true` if `int` is pending. Otherwise, returns `false`.
    pub fn is_pending(&self, int: Interrupt) -> bool {
        let int = int as u32;
        if int < 32 {
            self.registers.IRQ_pending_1.read() & (1 << int) != 0
        } else {
            self.registers.IRQ_pending_2.read() & (1 << (int - 32)) != 0
        }
    }
}
