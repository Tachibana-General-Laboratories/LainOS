#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct TrapFrame {
    pub tpidr: u64,
    pub sp: u64,
    pub spsr: u64,
    pub elr: u64,

    pub reserved: u64,

    pub x29: u64,
    pub x28: u64,
    pub x27: u64,
    pub x26: u64,
    pub x25: u64,
    pub x24: u64,
    pub x23: u64,
    pub x22: u64,
    pub x21: u64,
    pub x20: u64,
    pub x19: u64,
    pub x18: u64,
    pub x17: u64,
    pub x16: u64,
    pub x15: u64,
    pub x14: u64,
    pub x13: u64,
    pub x12: u64,
    pub x11: u64,
    pub x10: u64,
    pub x9: u64,
    pub x8: u64,
    pub x7: u64,
    pub x6: u64,
    pub x5: u64,
    pub x4: u64,
    pub x3: u64,
    pub x2: u64,
    pub x1: u64,

    // TODO: pub q31_q0: [u128; 32],

    pub x30: u64, // lr
    pub x0: u64,
}
