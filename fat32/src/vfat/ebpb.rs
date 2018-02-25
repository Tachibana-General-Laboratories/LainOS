use std::fmt;

use traits::BlockDevice;
use vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    bpb: [u8; 36],
    sec: u32,
    flags: u16,
    version: u16,
    root: u32,
    fs_info: u16,
    boot_backup: u16,
    _reserved: [u8; 12],
    drive: u8,
    _reserved_nt: u8,
    sign: u8,
    volid: u32,
    label: [u8; 11],
    sysid: [u8; 8],
    boot: [u8; 420],
    signature: [u8; 2],
}

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(
        mut device: T,
        sector: u64
    ) -> Result<BiosParameterBlock, Error> {
        use util::SliceExt;

        let mut buf = [0u8; 512];
        if let Err(err) = device.read_sector(sector, &mut buf[..]) {
            return Err(Error::Io(err))
        }
        let r: Self = unsafe { ::std::mem::transmute(buf) };

        if r.signature[0] != 0x55 || r.signature[1] != 0xAA {
            return Err(Error::BadSignature);
        }

        Ok(r)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!("BiosParameterBlock::debug()")
    }
}
