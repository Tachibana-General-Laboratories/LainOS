use std::collections::VecDeque;

use mutex::Mutex;
use process::{Process, State, Id};
use traps::TrapFrame;

/// The `tick` time.
// FIXME: When you're ready, change this to something more reasonable.
pub const TICK: u32 = 2 * 1000 * 1000;

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

extern "C" {
    fn el0_main();
}

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> Self {
        GlobalScheduler(Mutex::new(None))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.0.lock().as_mut().expect("scheduler uninitialized").add(process)
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::switch()`.
    #[must_use]
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        self.0.lock().as_mut().expect("scheduler uninitialized").switch(new_state, tf)
    }

    /// Initializes the scheduler and starts executing processes in user space
    /// using timer interrupt based preemptive scheduling. This method should
    /// not return under normal conditions.
    pub fn start(&self) -> ! {
        let mut process = Process::new().unwrap();
        process.trap_frame.elr = (el0_main as *const ()) as u64;
        process.trap_frame.sp = process.stack.top().as_u64();

        let trap_frame = {
            let p = &*process.trap_frame;
            let frame = p as *const TrapFrame as u64;
            ::core::mem::forget(p);
            frame
        };

        *self.0.lock() = Some(Scheduler::new());
        self.add(process);

        unsafe {
            asm!("
            mov x0, $0
            mov SP, x0
            bl  context_restore

            ldr x0, =_stack_core0_el1
            mov SP, x0
            mov x0, #0
            mov x30, #0
            eret
            "
            :: "r"(trap_frame)
            :: "volatile");
        }
        unreachable!("goto EL0")
    }
}

#[derive(Debug)]
struct Scheduler {
    processes: VecDeque<Process>,
    current: Option<Id>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Self {
        Self {
            processes: VecDeque::new(),
            current: None,
            last_id: None,
        }
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// If this is the first process added, it is marked as the current process.
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        self.last_id = if self.last_id.is_none() {
            process.trap_frame.tpidr = 1;
            self.current = process.id();
            self.processes.push_back(process);
            process.id()
        } else {
            unimplemented!("Scheduler::add")
        };

        self.last_id
    }

    /// Sets the current process's state to `new_state`, finds the next process
    /// to switch to, and performs the context switch on `tf` by saving `tf`
    /// into the current process and restoring the next process's trap frame
    /// into `tf`. If there is no current process, returns `None`. Otherwise,
    /// returns `Some` of the process ID that was context switched into `tf`.
    ///
    /// This method blocks until there is a process to switch to, conserving
    /// energy as much as possible in the interim.
    fn switch(&mut self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        /*
        let current = self.processes.pop_front();
        match current {
            Some(process) => {
                process.state = new_state;
                *process.trap_frame = *tf;

                self.processes.push_back(process);
            }
            None => None,
        }
        */
        unimplemented!("Scheduler::switch")
    }
}
