use core::{mem, fmt};
use core::num::NonZeroU64;
use traps::TrapFrame;
use process::{State, Stack};
use alloc::boxed::Box;

use vm::{self, Memory, Area, Prot};
use pi::common::IO_BASE_RAW;

use allocator::safe_box;

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
    // The memory allocation used for the process's stack.
    //pub stack: Stack,
    /// The scheduling state of the process.
    pub state: State,
    /// The virtual memory of the process.
    pub mm: Box<Memory>,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> Option<Self> {
        let stack_bottom = 4 * 0x4000_0000;
        let stack_top = stack_bottom + Stack::SIZE;

        let mut trap_frame: Box<TrapFrame> = safe_box(unsafe { mem::zeroed() })?;

        // create MMU translation tables
        let mut mm = Memory::new()?;

        let start = vm::kernel_start().as_usize();
        let data = vm::kernel_data().as_usize();
        let end = vm::kernel_end().as_usize();

        // different for code and data
        mm.area_rx(Area::new(start, data).map_to(start.into()))?;
        mm.area_rw(Area::new(data, end).map_to(data.into()))?;
        // different attributes for device memory
        mm.area_dev(Area::new(IO_BASE_RAW, 0x4000_0000).map_to(IO_BASE_RAW.into()))?;
        // different for stack area
        mm.area_rw(Area::new(stack_bottom, stack_top).prot(Prot::RW))?;

        let state = State::Ready;
        trap_frame.sp = stack_top as u64;
        trap_frame.set_ttbr(0, mm.ttbr());
        //mem::forget(mm);
        Some(Self { trap_frame, state, mm })
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
            self.trap_frame.pid = id.as_u64();
        } else {
            self.trap_frame.pid = 0;
        }
    }

    pub fn id(&self) -> Option<Id> {
        Id::new(self.trap_frame.pid)
    }
}
