use pi::interrupt::Interrupt;

use traps::TrapFrame;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    match interrupt {
        Interrupt::Timer1 => {
            ::pi::timer::tick_in(::process::TICK);
        }
        _ => unimplemented!(),
    }
}
