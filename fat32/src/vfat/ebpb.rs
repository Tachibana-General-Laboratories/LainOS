use std::fmt;

use traits::BlockDevice;
use vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    pub ebxx90: [u8; 3],
    pub oem: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub num_reserved_sectors: u16,
    pub num_of_fats: u8,
    pub max_dir_entries: u16,
    pub total_logical_sectors: u16,
    pub fat_id: u8,
    pub num_sectors_per_fat: u16,
    pub num_sectors_per_track: u16,
    pub num_hs: u16,
    pub num_hidden_sectors: u32,
    pub total_sectors: u32,

    // ebpb
    pub sectors_per_fat: u32,
    pub flags: u16,
    pub version: u16,
    pub root_dir_cluster: u32,
    pub fs_info: u16,
    pub boot_backup: u16,
    pub _reserved: [u8; 12],
    pub drive: u8,
    pub _reserved_nt: u8,
    pub sign: u8,
    pub volid: u32,
    pub label: [u8; 11],
    pub sysid: [u8; 8],
    pub boot: [u8; 420],
    pub signature: [u8; 2],
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
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("eBPB")
            .field("sectors_per_fat", &self.sectors_per_fat)
            .finish()
    }
}

