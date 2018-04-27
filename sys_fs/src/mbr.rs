use std::{fmt, io};

use sys::fs::BlockDevice;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct CHS {
    address: [u8; 3],
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct PartitionEntry {
    pub boot_indicator: u8,
    pub starting: CHS,
    pub ptype: u8, // 0xB or 0xC
    pub ending: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    pub bootstrap: [u8; 436],
    pub uid: [u8; 10],
    pub table: [PartitionEntry; 4],
    pub signature: [u8; 2],
}

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut buf = [0u8; 512];
        if let Err(err) = device.read_sector(0, &mut buf[..]) {
            return Err(Error::Io(err))
        }
        let r: Self = unsafe { ::std::mem::transmute(buf) };

        if r.signature[0] != 0x55 || r.signature[1] != 0xAA {
            return Err(Error::BadSignature);
        }

        for i in 0..4 {
            let n = r.table[i].boot_indicator;
            if n != 0x00 && n != 0x80 {
                return Err(Error::UnknownBootIndicator(i as u8));
            }
        }

        Ok(r)
    }
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("MBR")
            .field("sign", &self.signature)
            .field("uid", &self.uid)
            .field("table", &self.table)
            .finish()
    }
}
