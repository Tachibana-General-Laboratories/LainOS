use vm::{PhysicalAddr, LOWER_SPACE_MASK};

pub type EntryFn = unsafe extern "C" fn () -> !;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct TrapFrame {
    pub elr: u64,
    pub spsr: u64,
    pub sp: u64,
    pub pid: u64,

    pub ttbr: u64,
    pub reserved_1: u64,

    // TODO: pub q31_q0: [u128; 32],

    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,

    pub reserved_2: u64,

    pub x30: u64, // lr
    pub x0: u64,
}

impl TrapFrame {
    pub fn set_elr(&mut self, entry: EntryFn) {
        let entry = (entry as usize & LOWER_SPACE_MASK) as u64;
        assert_eq!(entry % 4, 0, "PC must be proprely aligned");
        self.elr = entry;
    }

    pub fn set_ttbr(&mut self, asid: u16, addr: PhysicalAddr) {
        let asid = (asid as u64) << 48;
        self.ttbr = asid | addr.as_u64() | 1;
    }
}
