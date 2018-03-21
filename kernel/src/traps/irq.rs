use pi::interrupt::Interrupt;
use pi::timer::tick_in;
use traps::TrapFrame;
use process::{State, TICK};
use SCHEDULER;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    match interrupt {
        Interrupt::Timer1 => {
            ::console::kprintln!("timer");
            tick_in(TICK);
            SCHEDULER.switch(State::Ready, tf);
        }
        _ => unimplemented!(),
    }
}
