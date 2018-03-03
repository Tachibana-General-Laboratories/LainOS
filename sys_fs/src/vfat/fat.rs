use std::fmt;
use vfat::*;

use self::Status::*;

#[derive(Debug, PartialEq)]
pub enum Status {
    /// The FAT entry corresponds to an unused (free) cluster.
    Free,
    /// The FAT entry/cluster is reserved.
    Reserved,
    /// The FAT entry corresponds to a valid data cluster. The next cluster in
    /// the chain is `Cluster`.
    Data(Cluster),
    /// The FAT entry corresponds to a bad (disk failed) cluster.
    Bad,
    /// The FAT entry corresponds to a valid data cluster. The corresponding
    /// cluster is the last in its chain.
    Eoc(u32)
}

#[repr(C, packed)]
pub struct FatEntry(pub u32);

impl FatEntry {
    /// Returns the `Status` of the FAT entry `self`.
    pub fn status(&self) -> Status {
        let v = self.0 & 0x0FFF_FFFF;
        match v {
            0x000_0000 => Free,
            0x000_0001 => Reserved,
            0x000_0002...0xFFF_FFEF => Data(Cluster::from(v)),
            0xFFF_FFF0...0xFFF_FFF5 => Reserved,
            0xFFF_FFF6 => Reserved,
            0xFFF_FFF7 => Bad,
            0xFFF_FFF8...0xFFF_FFFF => Eoc(v),
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for FatEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FatEntry")
            .field(&self.0)
            .field(&self.status())
            .finish()
    }
}
