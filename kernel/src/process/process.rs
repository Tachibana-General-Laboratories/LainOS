use core::mem;
use core::nonzero::NonZero;
use traps::TrapFrame;
use process::{State, Stack};

/// Type alias for the type of a process ID.
pub type Id = NonZero<u64>;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub trap_frame: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    pub stack: Stack,
    /// The scheduling state of the process.
    pub state: State,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> Option<Self> {
        Stack::new().map(|stack| {
            let state = State::Ready;
            let mut trap_frame = Box::new(unsafe { mem::zeroed() });
            Self { trap_frame, stack, state }
        })
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occured. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        let p = self as *mut Self;
        let ret = match self.state {
            State::Ready => true,
            State::Running => false,
            State::Waiting(ref mut f) => {
                let p = unsafe { &mut *p };
                let r = f(p);
                mem::forget(p);
                r
            }
        };
        if ret {
            self.state = State::Ready;
        }
        ret
    }

    pub fn set_id(&mut self, id: Option<Id>) {
        if let Some(id) = id {
            self.trap_frame.tpidr = id.get();
        } else {
            self.trap_frame.tpidr = 0;
        }
    }

    pub fn id(&self) -> Option<Id> {
        NonZero::new(self.trap_frame.tpidr)
    }
}
