use std::{io, fmt};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
//use fnv::*;

use sys::fs::BlockDevice;

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64
}

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool,
}

pub struct CachedDevice {
    device: Box<BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition,
}

impl CachedDevice {
    /// Creates a new `CachedDevice` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `n` _before_
    /// `partition.start` is made to physical sector `n`. Cached sectors before
    /// `partition.start` are the size of a physical sector. An access to a
    /// sector `n` at or after `partition.start` is made to the _logical_ sector
    /// `n - partition.start`. Cached sectors at or after `partition.start` are
    /// the size of a logical sector, `partition.sector_size`.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> Self
        where T: BlockDevice + 'static
    {
        assert!(partition.sector_size >= device.sector_size());

        Self {
            device: Box::new(device),
            cache: HashMap::default(),
            partition,
        }
    }

    /// Maps a user's request for a sector `virt` to the physical sector and
    /// number of physical sectors required to access `virt`.
    fn virtual_to_physical(&self, virt: u64) -> (u64, u64) {
        if self.device.sector_size() == self.partition.sector_size {
            (virt, 1)
        } else if virt < self.partition.start {
            (virt, 1)
        } else {
            let factor = self.partition.sector_size / self.device.sector_size();
            let logical_offset = virt - self.partition.start;
            let physical_offset = logical_offset * factor;
            let physical_sector = self.partition.start + physical_offset;
            (physical_sector, factor)
        }
    }

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        let (sector, factor) = self.virtual_to_physical(sector);

        let cap = self.sector_size() as usize;
        let entry = match self.cache.entry(sector) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().dirty = true;
                entry.into_mut()
            }
            Entry::Vacant(entry) => {
                let mut data = Vec::with_capacity(cap);
                for i in 0..factor {
                    self.device.read_all_sector(sector + i, &mut data)?;
                }
                entry.insert(CacheEntry { data, dirty: true })
            }
        };
        Ok(&mut entry.data)
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        let (sector, factor) = self.virtual_to_physical(sector);

        let cap = self.sector_size() as usize;
        let entry = match self.cache.entry(sector) {
            Entry::Occupied(mut entry) => {
                entry.into_mut()
            }
            Entry::Vacant(entry) => {
                let mut data = Vec::with_capacity(cap);
                for i in 0..factor {
                    self.device.read_all_sector(sector + i, &mut data)?;
                }
                entry.insert(CacheEntry { data, dirty: false })
            }
        };
        Ok(&entry.data)
    }

    pub fn drop_read_cache(&mut self) {
        self.cache.retain(|_, v| !v.dirty)
    }

    pub fn sync_sector(&mut self, sector: u64, remove: bool) -> io::Result<()> {
        let (sector, factor) = self.virtual_to_physical(sector);

        let chunk_size = self.device.sector_size() as usize;
        if let Entry::Occupied(mut entry) = self.cache.entry(sector) {
            {
                let entry = entry.get_mut();
                if entry.dirty {
                    let chunks = entry.data.chunks(chunk_size);
                    for (i, data) in chunks.enumerate() {
                        self.device.write_sector(sector + i as u64, data)?;
                    }
                    entry.dirty = false;
                }
            }
            if remove {
                entry.remove_entry();
            }
        }
        Ok(())
    }
}

impl BlockDevice for CachedDevice {
    fn sector_size(&self) -> u64 {
        self.partition.sector_size
    }
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        let sector = self.get(n)?;
        let len = buf.len().min(sector.len());
        buf[..len].copy_from_slice(&sector[..len]);
        Ok(len)
    }
    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        let sector = self.get_mut(n)?;
        let len = buf.len().min(sector.len());
        sector[..len].copy_from_slice(&buf[..len]);
        Ok(len)
    }
}

impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedDevice")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
