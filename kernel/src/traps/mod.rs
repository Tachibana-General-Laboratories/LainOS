mod irq;
mod trap_frame;
mod syndrome;
mod syscall;

use pi::interrupt::{Controller, Interrupt};

pub use self::trap_frame::TrapFrame;
pub use self::syscall::Error;

use console::kprintln;
use self::syndrome::Syndrome;
use self::irq::handle_irq;
use self::syscall::handle_syscall;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    use self::Kind::*;
    use self::Source::*;
    use self::Interrupt::*;

    let syndrome = Syndrome::from(esr as u32);
    let ctl = Controller::new();
    if info.source == LowerAArch64 {
        match info.kind {
            Synchronous => {
                match syndrome {
                    Syndrome::Svc(num) => return handle_syscall(num, tf),
                    Syndrome::Brk(num) => {
                        kprintln!("--- BRK {:?} at {:08X}", num, tf.elr);
                        tf.elr += 4;
                        return;
                    }
                    _ => (),
                }
            }

            Irq if ctl.is_pending(Timer1) => return handle_irq(Timer1, tf),
            Irq if ctl.is_pending(Timer3) => return handle_irq(Timer3, tf),
            Irq if ctl.is_pending(Usb   ) => return handle_irq(Usb   , tf),
            Irq if ctl.is_pending(Gpio0 ) => return handle_irq(Gpio0 , tf),
            Irq if ctl.is_pending(Gpio1 ) => return handle_irq(Gpio1 , tf),
            Irq if ctl.is_pending(Gpio2 ) => return handle_irq(Gpio2 , tf),
            Irq if ctl.is_pending(Gpio3 ) => return handle_irq(Gpio3 , tf),
            Irq if ctl.is_pending(Uart  ) => return handle_irq(Uart  , tf),

            _ => (),
        }
    }

    /*
    if info.source == CurrentSpElx && info.kind == Synchronous {
        match syndrome {
            Syndrome::Brk(num) => {
                kprintln!("!!! BRK EL1 {:?} at {:08X}", num, tf.elr);
                tf.elr += 4;
                return;
            }
            _ => (),
        }
    }
    */

    panic!("IT'S A TRAP: {:?} {:?} {:?} esr: {:08x}\n{:?}",
           info.source, info.kind, syndrome, esr, tf);
}
