use traps::TrapFrame;

use console::kprintln;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sleep(ms: u32, tf: &mut TrapFrame) {
    unimplemented!("syscall: sleep()")
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    kprintln!("--- SYSCALL {:?}, x0-3: {} {} {} {}",
                num, tf.x0, tf.x1, tf.x2, tf.x3);

    //kprintln!("--- SYSCALL {:?}, {:?}", num, tf);
    tf.x0 = num as u64;
}
