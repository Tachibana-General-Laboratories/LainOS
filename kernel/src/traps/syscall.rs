use traps::TrapFrame;
use console::kprintln;
use process::{State, Process};
use pi::timer::current_time;
use SCHEDULER;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sleep(ms: u32, tf: &mut TrapFrame) {
    let exit_time = current_time() + 1000 * ms as u64;
    let f = Box::new(move |p: &mut Process| {
        let elapsed = exit_time.checked_sub(current_time());
        if let Some(elapsed) = elapsed {
            p.trap_frame.x0 = (elapsed / 1000) & 0xFFFF_FFFF;
            true
        } else {
            false
        }
    });
    SCHEDULER.switch(State::Waiting(f), tf);
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num {
        1 => sleep(tf.x0 as u32, tf),
        _ => {
            kprintln!("--- SYSCALL {:?}, x0-3: {} {} {} {}", num, tf.x0, tf.x1, tf.x2, tf.x3);
            //kprintln!("--- SYSCALL {:?}, {:?}", num, tf);
            tf.x0 = num as u64;
        }
    }
}
