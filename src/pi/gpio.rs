use std::marker::PhantomData;

use super::{IO_BASE, states};
use volatile::prelude::*;
use volatile::{Volatile, WriteVolatile, ReadVolatile, Reserved};

/// An alternative GPIO function.
#[repr(u8)]
pub enum Function {
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct Registers {
    pub FSEL: [Volatile<u32>; 6],
    __r0: Reserved<u32>,
    pub SET: [WriteVolatile<u32>; 2],
    __r1: Reserved<u32>,
    pub CLR: [WriteVolatile<u32>; 2],
    __r2: Reserved<u32>,
    pub LEV: [ReadVolatile<u32>; 2],
    __r3: Reserved<u32>,
    pub EDS: [Volatile<u32>; 2],
    __r4: Reserved<u32>,
    pub REN: [Volatile<u32>; 2],
    __r5: Reserved<u32>,
    pub FEN: [Volatile<u32>; 2],
    __r6: Reserved<u32>,
    pub HEN: [Volatile<u32>; 2],
    __r7: Reserved<u32>,
    pub LEN: [Volatile<u32>; 2],
    __r8: Reserved<u32>,
    pub AREN: [Volatile<u32>; 2],
    __r9: Reserved<u32>,
    pub AFEN: [Volatile<u32>; 2],
    __r10: Reserved<u32>,
    pub PUD: Volatile<u32>,
    pub PUDCLK: [Volatile<u32>; 2],
}

/// Possible states for a GPIO pin.
states! {
    Uninitialized, Input, Output, Alt
}

/// A GPIP pin in state `State`.
///
/// The `State` generic always corresponds to an uninstantiatable type that is
/// use solely to mark and track the state of a given GPIO pin. A `Gpio`
/// structure starts in the `Uninitialized` state and must be transitions into
/// one of `Input`, `Output`, or `Alt` via the `into_input`, `into_output`, and
/// `into_alt` methods before it can be used.
pub struct Gpio<State> {
    pin: u8,
    registers: &'static mut Registers,
    _state: PhantomData<State>
}

/// The base address of the `GPIO` registers.
pub const GPIO_BASE: usize = IO_BASE + 0x200000;

impl<T> Gpio<T> {
    /// Transitions `self` to state `S`, consuming `self` and returning a new
    /// `Gpio` instance in state `S`. This method should _never_ be exposed to
    /// the public!
    #[inline(always)]
    fn transition<S>(self) -> Gpio<S> {
        Gpio {
            pin: self.pin,
            registers: self.registers,
            _state: PhantomData
        }
    }
}

impl Gpio<Uninitialized> {
    /// Returns a new `GPIO` structure for pin number `pin`.
    ///
    /// # Panics
    ///
    /// Panics if `pin` > `53`.
    pub fn new(pin: u8) -> Self {
        if pin > 53 {
            panic!("Gpio::new(): pin {} exceeds maximum of 53", pin);
        }
        Self {
            registers: unsafe { &mut *(GPIO_BASE as *mut Registers) },
            pin: pin,
            _state: PhantomData
        }
    }

    /// Enables the alternative function `function` for `self`. Consumes self
    /// and returns a `Gpio` structure in the `Alt` state.
    pub fn into_alt(self, function: Function) -> Gpio<Alt> {
        self.transition()
    }

    /// Sets this pin to be an _output_ pin. Consumes self and returns a `Gpio`
    /// structure in the `Output` state.
    pub fn into_output(self) -> Gpio<Output> {
        self.into_alt(Function::Output).transition()
    }

    /// Sets this pin to be an _input_ pin. Consumes self and returns a `Gpio`
    /// structure in the `Input` state.
    pub fn into_input(self) -> Gpio<Input> {
        self.into_alt(Function::Input).transition()
    }
}

impl Gpio<Output> {
    /// Sets (turns on) the pin.
    pub fn set(&mut self) {
        unimplemented!()
    }

    /// Clears (turns off) the pin.
    pub fn clear(&mut self) {
        unimplemented!()
    }
}

impl Gpio<Input> {
    /// Reads the pin's value. Returns `true` if the level is high and `false`
    /// if the level is low.
    pub fn level(&mut self) -> bool {
        unimplemented!()
    }
}
