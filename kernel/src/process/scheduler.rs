use alloc::VecDeque;
use core::num::NonZeroU64;
use core::mem;

use sys::Mutex;
use process::{Process, State, Id};
use traps::TrapFrame;
use aarch64::wait_for_interrupt;

/// The `tick` time.
// FIXME: When you're ready, change this to something more reasonable.
pub const TICK: u32 = 2 * 1000 * 1000;

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

extern "C" {
    fn el0_init() -> !;
    fn el0_other() -> !;
}

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> Self {
        GlobalScheduler(Mutex::new(None))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.0.lock().unwrap().as_mut().expect("scheduler uninitialized").add(process)
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::switch()`.
    #[must_use]
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        self.0.lock().unwrap().as_mut().expect("scheduler uninitialized").switch(new_state, tf)
    }

    /// Initializes the scheduler and starts executing processes in user space
    /// using timer interrupt based preemptive scheduling. This method should
    /// not return under normal conditions.
    pub fn start(&self) -> ! {
        *self.0.lock().unwrap() = Some(Scheduler::new());

        let tf = {
            let mut init = Process::with_entry(el0_init).unwrap();
            let tf = init.tf_u64();
            self.add(init).expect("add proc 'init'");
            self.add(Process::with_entry(el0_other).unwrap()).expect("add proc 'other'");
            tf
        };

        unsafe {
            asm!("
            mov SP, $0
            bl  context_restore
            ldr x0, =_stack_core0_el1
            mov SP, x0
            mov x0, #0
            mov x30, #0
            eret
            "
            :: "r"(tf)
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
         let id = match self.last_id {
            Some(id) => {
                let id = id.get().checked_add(1)?;
                unsafe { NonZeroU64::new_unchecked(id) }
            }
            None => {
                let id = unsafe { NonZeroU64::new_unchecked(1) };
                process.state = State::Running;
                self.current = Some(id);
                id
            }
        };

        process.trap_frame.tpidr = id.get();
        self.processes.push_back(process);

        self.last_id = Some(id);
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
        if let Some(mut process) = self.processes.pop_front() {
            if process.id() == self.current {
                *process.trap_frame = *tf;
                process.state = new_state;
                self.current = None;
            }
            self.processes.push_back(process);
        }

        // TODO: improve WFI works

        loop {
            for _ in 0..self.processes.len() {
                let mut process = self.processes.pop_front().unwrap();

                if !process.is_ready() {
                    self.processes.push_back(process);
                    continue;
                }

                *tf = *process.trap_frame;
                self.current = process.id();
                self.processes.push_front(process);
                return self.current;
            }
            wait_for_interrupt();
        }
        None
    }
}
