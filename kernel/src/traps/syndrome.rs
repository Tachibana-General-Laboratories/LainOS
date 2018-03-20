#![deny(unreachable_pattern)]

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8)
}

impl From<u32> for Fault {
    fn from(val: u32) -> Self {
        use self::Fault::*;

        match (val & 0b111111) as u8 {
            0b000000 |
            0b000001 |
            0b000010 |
            0b000011 => AddressSize,

            0b000100 |
            0b000101 |
            0b000110 |
            0b000111 => Translation,

            0b001000 |
            0b001001 |
            0b001010 |
            0b001011 => AccessFlag,

            //0b001100 |
            0b001101 |
            0b001110 |
            0b001111 => Permission,

            0b100001 => Alignment,
            0b110000 => TlbConflict,

            fault => Other(fault),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    McrMrc,
    McrrMrrc,
    LdcStc,
    SimdFp,
    Vmrs,
    Mrrc,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort {
        kind: Fault,
        level: u8,
    },
    PCAlignmentFault,
    DataAbort {
        kind: Fault,
        level: u8
    },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32)
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Self {
        use self::Syndrome::*;

        let syndrome = esr >> 26;

        match syndrome {
            0b000000 => Unknown,
            0b000001 => WfiWfe,
            0b000011 => McrMrc,
            0b000100 | 0b000101 => McrrMrrc,
            0b000110 => LdcStc,
            0b000111 => SimdFp,
            0b001000 => Vmrs,
            0b001100 => Mrrc,
            0b001110 => IllegalExecutionState,

            0b010001 | 0b010101 => Svc(esr as u16),
            0b010010 | 0b010110 => Hvc(esr as u16),
            0b010011 | 0b010111 => Smc(esr as u16),
            0b011000 => MsrMrsSystem,

            0b101111 => SError,

            0b100000 | 0b100001 => InstructionAbort {
                kind: Fault::from(esr),
                level: (esr & 0b11) as u8,
            },
            0b100010 => PCAlignmentFault,
            0b100100 | 0b100101 => DataAbort {
                kind: Fault::from(esr),
                level: (esr & 0b11) as u8,
            },
            0b100110 => SpAlignmentFault,
            0b101000 | 0b101100 => TrappedFpu,
            0b110000 | 0b110001 => Breakpoint,
            0b110010 | 0b110011 => Step,
            0b110100 | 0b110101 => Watchpoint,

            0b111000 | 0b111100 => Brk(esr as u16),

            _ => Other(esr),
        }
    }
}
