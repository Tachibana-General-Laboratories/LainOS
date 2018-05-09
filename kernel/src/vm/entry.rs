use vm::PhysicalAddr;

#[repr(align(4096))]
pub struct Table {
    pub entries: [Entry; 512],
}

bitflags! {
    pub struct Entry: u64 {
        const VALID = 1;
        const PAGE = 0b11;
        const TABLE = 0b11;
        const BLOCK = 0b01;

        // 58-55 for software use
        const USER_MASK     = 0x0780_0000_0000_0000;
        const UPPER_MASK    = 0xFFF0_0000_0000_0000;
        const LOWER_MASK    = 0x0000_0000_0000_0FFC;
        const ADDRESS_MASK  = 0x0000_FFFF_FFFF_F000;

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors
        // 58-51 bits is ignored
        const NS_TABLE = 1 << 63;
        const AP_TABLE = 0b11 << 61;
        const XN_TABLE = 1 << 60;
        const UXN_TABLE = Self::XN_TABLE.bits;
        const PXN_TABLE = 1 << 59;

        const PBHA = 0b1111 << 59;

        const XN = 1 << 54;
        const UXN = Self::XN.bits;
        const PXN = 1 << 53;
        const CONTIGUOUS = 1 << 52;
        const DBM = 1 << 51;

        const N_G = 1 << 11;
        const AF = 1 << 10;
        const SH = 0b11 << 8;
        const AP = 0b11 << 6;
        const NS = 1 << 5;
        const ATTR_INDX = 0b11 << 2;

        const AP_RO = 1 << 7;
        const AP_EL0 = 1 << 6;

        const SH_OUTER = 0b10 << 8;
        const SH_INNER = 0b11 << 8;
    }
}

impl Entry {
    pub fn table(addr: PhysicalAddr) -> Self {
        Self::TABLE.with_address(addr)
    }

    pub fn page(addr: PhysicalAddr) -> Self {
        Self::PAGE.with_address(addr)
    }

    pub fn block(addr: PhysicalAddr) -> Self {
        Self::BLOCK.with_address(addr)
    }

    pub fn is_valid(self) -> bool {
        self.contains(Self::VALID)
    }

    pub fn with_valid(mut self, valid: bool) -> Self {
        self.set(Self::VALID, valid);
        self
    }

    pub fn is_block(self) -> bool {
        self.bits & 0b11 == 0b01
    }

    pub fn is_table(self) -> bool {
        self.bits & 0b11 == 0b11
    }

    pub fn user_data(self) -> u8 {
        ((self.bits >> 55) & 0b1111) as u8
    }
    pub fn with_user_data(mut self, data: u8) -> Self {
        self.bits |= ((data & 0b1111) as u64) << 55;
        self
    }

    pub fn with_attr_index(mut self, idx: u8) -> Self {
        self.bits |= ((idx & 0b11) as u64) << 2;
        self
    }

    pub fn address(self) -> PhysicalAddr {
        PhysicalAddr::from((self & Self::ADDRESS_MASK).bits as *mut u8)
    }
    pub fn with_address(mut self, addr: PhysicalAddr) -> Self {
        self.bits |= (addr.as_u64()) & Self::ADDRESS_MASK.bits;
        self
    }
}
