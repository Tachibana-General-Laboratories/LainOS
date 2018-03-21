use alloc::slice::from_raw_parts;
use alloc::str::from_utf8;

use traps::TrapFrame;
use console::kprintln;
use process::{State, Process};
use pi::timer::current_time;
use console::CONSOLE;
use SCHEDULER;


#[repr(u64)]
#[derive(Debug)]
pub enum Error {
    SyscallDoesNotExist = 1,
    Utf8 = 2,
    Io = 3,

    Other = 0xFFFF_FFFF,
}

impl From<u64> for Error {
    fn from(val: u64) -> Self {
        use self::Error::*;
        match val {
            1 => SyscallDoesNotExist,
            2 => Utf8,
            3 => Io,
            _ => Other,
        }
    }
}


// Syscall Convention:
//
// - System call n is invoked with svc #n.
// - Up to 7 parameters can be passed to a system call in registers x0...x6.
// - Up to 7 parameters can be returned from a system call in registers x0...x6.
// - Register x7 is used to indicate an error.
//     - If x7 is 0, there was no error.
//     - If x7 is 1, the system call does not exist.
//     - If x7 is any other value, it represents an error code specific to the system call.
// - All other registers and program state are preserved by the kernel.



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

pub fn print(s: *const u8, len: usize, tf: &mut TrapFrame) -> Result<(), Error> {
    use core::fmt::Write;

    let s = from_utf8(unsafe { from_raw_parts(s, len) })
        .map_err(|_| Error::Utf8)?;
    CONSOLE.lock().write_str(s)
        .map_err(|_| Error::Io)
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    tf.x7 = 0;
    match num {
        1 => sleep(tf.x0 as u32, tf),
        2 => {
            if let Err(err) = print(tf.x0 as *const u8, tf.x1 as usize, tf) {
                tf.x7 = err as u64;
            }
        }
        _ => {
            kprintln!("--- SYSCALL does not exists {:?}, x0-3: {} {} {} {}", num, tf.x0, tf.x1, tf.x2, tf.x3);
            tf.x0 = num as u64;
            tf.x7 = Error::SyscallDoesNotExist as u64;
        }
    }
}
