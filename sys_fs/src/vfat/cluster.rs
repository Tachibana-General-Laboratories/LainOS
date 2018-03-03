use vfat::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

impl Cluster {
    pub fn sector(self, sectors_per_cluster: u8) -> u64 {
        let cluster = self.0 as u64;
        //cluster * sectors_per_cluster as u64
        (cluster - 2) * sectors_per_cluster as u64
    }
    pub fn fat_offset(self) -> u64 {
        4 * self.0 as u64
    }
}
