use core::{mem, fmt};
use core::num::NonZeroU64;
use traps::TrapFrame;
use process::{State, Stack};
use alloc::boxed::Box;

/// Type alias for the type of a process ID.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id {
    id: NonZeroU64,
}

impl Id {
    pub fn one() -> Self {
        Self { id: unsafe { NonZeroU64::new_unchecked(1) } }
    }

    pub fn new(id: u64) -> Option<Self> {
        Some(Self { id: NonZeroU64::new(id)? })
    }

    pub fn next(self) -> Option<Self> {
        let id = self.id.get().checked_add(1)?;
        let id = unsafe { NonZeroU64::new_unchecked(id) };
        Some(Self { id })
    }

    pub fn as_u64(self) -> u64 {
        self.id.get()
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id({})", self.id.get())
    }
}

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
            let mut trap_frame: Box<TrapFrame> = Box::new(unsafe { mem::zeroed() });
            trap_frame.sp = stack.top().as_u64();
            Self { trap_frame, stack, state }
        })
    }
    pub fn with_entry(entry: unsafe extern "C" fn () -> !) -> Option<Self> {
        Self::new().map(|mut p| {
            p.trap_frame.set_elr(entry);
            p
        })
    }

    pub fn tf_u64(&mut self) -> u64 {
        let p = &*self.trap_frame;
        let tf = p as *const TrapFrame as u64;
        mem::forget(p);
        tf
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
        let state = mem::replace(&mut self.state, State::Ready);
        match state {
            State::Ready => true,
            State::Running => {
                mem::replace(&mut self.state, State::Running);
                false
            }
            State::Exit(code) => {
                mem::replace(&mut self.state, State::Exit(code));
                false
            }
            State::Waiting(mut f) => {
                let ret = f(self);
                mem::replace(&mut self.state, State::Waiting(f));
                ret
            }
        }
    }

    pub fn set_id(&mut self, id: Option<Id>) {
        if let Some(id) = id {
            self.trap_frame.tpidr = id.as_u64();
        } else {
            self.trap_frame.tpidr = 0;
        }
    }

    pub fn id(&self) -> Option<Id> {
        Id::new(self.trap_frame.tpidr)
    }
}
