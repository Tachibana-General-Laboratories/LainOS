mod irq;
mod trap_frame;
mod syndrome;
mod syscall;

use pi::interrupt::{Controller, Interrupt};

pub use self::trap_frame::TrapFrame;

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
    let syndrome = Syndrome::from(esr as u32);

    match (info.kind, info.source, syndrome) {
        (Kind::Synchronous, Source::LowerAArch64, Syndrome::Svc(num)) => {
            //handle_syscall(num, tf);
            kprintln!("--- SYSCALL {:?}", num);
            tf.x0 = num as u64;
            return;
        }
        (Kind::Synchronous, _, Syndrome::Brk(num)) => {
            kprintln!("--- BRK {:?}", num);
            tf.elr += 4;
            return;
        }
        (Kind::Irq, _, _) => {
            //handle_syscall(num, tf);
            panic!("--- IRQ {:?} esr: {:08X}", info.source, esr);
            //handle_irq();
            return;
        }
        _ => {
            panic!("IT'S A TRAP: {:?} {:?} {:?}", info, syndrome, tf);
        }
    }
}
