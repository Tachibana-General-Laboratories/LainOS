use core::fmt;

/// Returns the current stack pointer.
#[inline(always)]
pub fn sp() -> *const u8 {
    let ptr: usize;
    unsafe {
        asm!("mov $0, sp" : "=r"(ptr));
    }
    ptr as *const u8
}

/// Returns the current exception level.
///
/// # Safety
/// This function should only be called when EL is >= 1.
#[inline(always)]
pub unsafe fn current_el() -> u8 {
    let el_reg: u64;
    asm!("mrs $0, CurrentEL" : "=r"(el_reg));
    ((el_reg & 0b1100) >> 2) as u8
}

/// Returns the SPSel value.
#[inline(always)]
pub fn sp_sel() -> u8 {
    let ptr: u32;
    unsafe {
        asm!("mrs $0, SPSel" : "=r"(ptr));
    }
    (ptr & 1) as u8
}

/// Returns the core currently executing.
///
/// # Safety
///
/// This function should only be called when EL is >= 1.
pub unsafe fn affinity() -> usize {
    let x: usize;
    asm!("mrs     $0, mpidr_el1
          and     $0, $0, #3"
          : "=r"(x));
    x
}

/// A NOOP that won't be optimized out.
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

pub fn wait_for_interrupt() {
    unsafe {
        asm!("wfi" :::: "volatile");
    }
}

/// Arch64 Auxiliary Feature Register
pub fn aa64afr_el1() -> [u64; 2] {
    let r0: u64;
    let r1: u64;
    unsafe {
        asm!("
            mrs $0, ID_AA64AFR0_EL1
            mrs $1, ID_AA64AFR1_EL1
            "
            : "=r"(r0), "=r"(r1)
            : : : "volatile");
    }
    [r0, r1]
}

/// Arch64 Debug Feature Register
pub fn aa64dfr_el1() -> [u64; 2] {
    let r0: u64;
    let r1: u64;
    unsafe {
        asm!("
            mrs $0, ID_AA64DFR0_EL1
            mrs $1, ID_AA64DFR1_EL1
            "
            : "=r"(r0), "=r"(r1)
            : : : "volatile");
    }
    [r0, r1]
}

/// Istruction Set Attribute Register
pub fn aa64isar_el1() -> [u64; 2] {
    let r0: u64;
    let r1: u64;
    unsafe {
        asm!("
            mrs $0, ID_AA64ISAR0_EL1
            mrs $1, ID_AA64ISAR1_EL1
            "
            : "=r"(r0), "=r"(r1)
            : : : "volatile");
    }
    [r0, r1]
}

/// Arch64 Memory Model Feature Register
pub fn aa64mmfr_el1() -> [u64; 2] {
    let r0: u64;
    let r1: u64;
    // only in ARMv8.2
    //let r2: u64;
    //mrs $2, ID_AA64MMFR2_EL1
    unsafe {
        asm!("
            mrs $0, ID_AA64MMFR0_EL1
            mrs $1, ID_AA64MMFR1_EL1
            "
            : "=r"(r0), "=r"(r1)
            : : : "volatile");
    }
    [r0, r1]
}

/// AArch64 Processor Feature Register
pub fn aa64pfr_el1() -> [u64; 2] {
    let r0: u64;
    let r1: u64;
    unsafe {
        asm!("
            mrs $0, ID_AA64PFR0_EL1
            mrs $1, ID_AA64PFR1_EL1
            "
            : "=r"(r0), "=r"(r1)
            : : : "volatile");
    }
    [r0, r1]
}

pub struct MMFR {
    pub raw: [u64; 2],
}

impl MMFR {
    pub fn new() -> Self {
        let raw = aa64mmfr_el1();
        Self { raw }
    }

    /// Support for 4KB memory translation granule size.
    pub fn support_4kb_granulate(&self) -> bool {
        (self.raw[0] >> 28) & 0b1111 == 0b0000
    }

    /// Support for 16KB memory translation granule size.
    pub fn support_16kb_granulate(&self) -> bool {
        (self.raw[0] >> 20) & 0b1111 == 0b0001
    }

    /// Support for 64KB memory translation granule size.
    pub fn support_64kb_granulate(&self) -> bool {
        (self.raw[0] >> 24) & 0b1111 == 0b0000
    }

    pub fn physical_address_range(&self) -> PhysicalAddressRange {
        match self.raw[0] & 0b1111 {
            0b0000 => PhysicalAddressRange::R32,
            0b0001 => PhysicalAddressRange::R36,
            0b0010 => PhysicalAddressRange::R40,
            0b0011 => PhysicalAddressRange::R42,
            0b0100 => PhysicalAddressRange::R44,
            0b0101 => PhysicalAddressRange::R48,
            0b0110 => PhysicalAddressRange::R52,
            other => panic!("other: {}", other),
        }
    }
}

pub struct AArch64 {
    pub afr: [u64; 2],
    pub dfr: [u64; 2],
    pub isar: [u64; 2],
    pub mmfr: [u64; 2],
    pub pfr: [u64; 2],
}

impl AArch64 {
    pub fn new() -> Self {
        let afr = aa64afr_el1();
        let dfr = aa64dfr_el1();
        let isar = aa64isar_el1();
        let mmfr = aa64mmfr_el1();
        let pfr = aa64pfr_el1();
        Self { afr, dfr, isar, mmfr, pfr }
    }


    /// Support for 4KB memory translation granule size.
    pub fn support_4kb_granulate(&self) -> bool {
        (self.mmfr[0] >> 28) & 0b1111 == 0b0000
    }

    /// Support for 16KB memory translation granule size.
    pub fn support_16kb_granulate(&self) -> bool {
        (self.mmfr[0] >> 20) & 0b1111 == 0b0001
    }

    /// Support for 64KB memory translation granule size.
    pub fn support_64kb_granulate(&self) -> bool {
        (self.mmfr[0] >> 24) & 0b1111 == 0b0000
    }

    pub fn physical_address_range(&self) -> PhysicalAddressRange {
        match self.mmfr[0] & 0b1111 {
            0b0000 => PhysicalAddressRange::R32,
            0b0001 => PhysicalAddressRange::R36,
            0b0010 => PhysicalAddressRange::R40,
            0b0011 => PhysicalAddressRange::R42,
            0b0100 => PhysicalAddressRange::R44,
            0b0101 => PhysicalAddressRange::R48,
            0b0110 => PhysicalAddressRange::R52,
            other => panic!("other: {}", other),
        }
    }
}

impl fmt::Debug for AArch64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "AArch64 ID registers:")?;
        writeln!(f, "     AFR: {:064b} {:064b}", self.afr[0], self.afr[1])?;
        writeln!(f, "     DFR: {:064b} {:064b}", self.dfr[0], self.dfr[1])?;
        writeln!(f, "    ISAR: {:064b} {:064b}", self.isar[0], self.isar[1])?;
        //writeln!(f, "  MMFR: {:064b} {:064b} {:064b}", self.mmfr[0], self.mmfr[1], self.mmfr[2])?;
        writeln!(f, "    MMFR: {:064b} {:064b}", self.mmfr[0], self.mmfr[1])?;
        writeln!(f, "     PFR: {:064b} {:064b}", self.pfr[0], self.pfr[1])?;

        writeln!(f, "  Support Granulate:{}{}{} with {:?}",
            if self.support_4kb_granulate() { " 4kb" } else { "" },
            if self.support_16kb_granulate() { " 16kb" } else { "" },
            if self.support_64kb_granulate() { " 64kb" } else { "" },
            self.physical_address_range(),
        )?;

        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhysicalAddressRange {
    R32 = 0b0000, // 32 bits, 4GB
    R36 = 0b0001, // 36 bits, 64GB
    R40 = 0b0010, // 40 bits, 1TB
    R42 = 0b0011, // 42 bits, 4TB
    R44 = 0b0100, // 44 bits, 16TB
    R48 = 0b0101, // 48 bits, 256TB
    R52 = 0b0110, // 52 bits, 4PB (only in ARMv8.2-LPA)
}

impl PhysicalAddressRange {
    pub fn raw(&self) -> usize {
        match self {
            PhysicalAddressRange::R32 => 0b0000,
            PhysicalAddressRange::R36 => 0b0001,
            PhysicalAddressRange::R40 => 0b0010,
            PhysicalAddressRange::R42 => 0b0011,
            PhysicalAddressRange::R44 => 0b0100,
            PhysicalAddressRange::R48 => 0b0101,
            PhysicalAddressRange::R52 => 0b0110,
        }
    }
}

impl fmt::Debug for PhysicalAddressRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhysicalAddressRange::R32 => write!(f, "PhysicalAddressRange(32 bits, 4GB)"),
            PhysicalAddressRange::R36 => write!(f, "PhysicalAddressRange(36 bits, 64GB)"),
            PhysicalAddressRange::R40 => write!(f, "PhysicalAddressRange(40 bits, 1TB)"),
            PhysicalAddressRange::R42 => write!(f, "PhysicalAddressRange(42 bits, 4TB)"),
            PhysicalAddressRange::R44 => write!(f, "PhysicalAddressRange(44 bits, 16TB)"),
            PhysicalAddressRange::R48 => write!(f, "PhysicalAddressRange(48 bits, 256TB)"),
            PhysicalAddressRange::R52 => write!(f, "PhysicalAddressRange(52 bits, 4PB)"),
        }
    }
}

pub struct MemoryAttributeIndirectionRegister {
    pub raw: [u8; 8],
}

impl MemoryAttributeIndirectionRegister {
    pub fn el1() -> Self {
        let raw = mair_el1();
        Self { raw }
    }

    pub fn write_el1(&self) {
        set_mair_el1(self.raw)
    }
}

impl fmt::Debug for MemoryAttributeIndirectionRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MAIR[")?;
        for (i, v) in self.raw.iter().cloned().enumerate() {
            if i != 7 {
                write!(f, "{:04b}_{:04b}, ", (v >> 4) & 0b1111, v & 0b1111)?;
            } else {
                write!(f, "{:04b}_{:04b}", (v >> 4) & 0b1111, v & 0b1111)?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

/// Provides the memory attribute encodings corresponding
/// to the possible AttrIndx values in a Long-descriptor format
/// translation table entry for stage 1 translations at EL1.
pub fn mair_el1() -> [u8; 8] {
    let r: u64;
    unsafe {
        asm!("mrs $0, MAIR_EL1": "=r"(r) : : : "volatile");
    }
    u64_to_u8arr(r)
}

pub fn set_mair_el1(r: [u8; 8]) {
    let r: u64 = u8arr_to_u64(r);
    unsafe {
        asm!("mr MAIR_EL1, $0" :: "r"(r) :: "volatile");
    }
}

enum Outer {
    WriteThroughTransient,
    NonCacheable,
    WriteBackTransient,
    WriteThroughNonTransient,
    WriteBackNonTransient,
}

fn u64_to_u8arr(r: u64) -> [u8; 8] {
    [
        0xFF & (r >>  0) as u8,
        0xFF & (r >>  8) as u8,
        0xFF & (r >> 16) as u8,
        0xFF & (r >> 24) as u8,
        0xFF & (r >> 32) as u8,
        0xFF & (r >> 40) as u8,
        0xFF & (r >> 48) as u8,
        0xFF & (r >> 56) as u8,
    ]
}

fn u8arr_to_u64(r: [u8; 8]) -> u64 {
    ((r[0] as u64) <<  0) |
    ((r[1] as u64) <<  8) |
    ((r[2] as u64) << 16) |
    ((r[3] as u64) << 24) |
    ((r[4] as u64) << 32) |
    ((r[5] as u64) << 40) |
    ((r[6] as u64) << 48) |
    ((r[7] as u64) << 56)
}
