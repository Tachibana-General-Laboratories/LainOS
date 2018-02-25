use std::{io, fmt};
use std::collections::HashMap;

use traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool
}

pub struct CachedDevice {
    device: Box<BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
}

impl CachedDevice {
    /// Creates a new `CachedDevice` that transparently caches sectors from
    /// `device`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    pub fn new<T: BlockDevice + 'static>(device: T) -> Self {
        Self {
            device: Box::new(device),
            cache: HashMap::new()
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
        use std::collections::hash_map::Entry;
        let entry = match self.cache.entry(sector) {
            Entry::Occupied(mut entry) => {
                {
                    let e = entry.get_mut();
                    if e.dirty {
                        e.data.clear();
                        self.device.read_all_sector(sector, &mut e.data)?;
                        e.dirty = false;
                    }
                }
                entry.into_mut()
            }
            Entry::Vacant(entry) => {
                let mut data = Vec::with_capacity(self.device.sector_size() as usize);
                self.device.read_all_sector(sector, &mut data)?;
                entry.insert(CacheEntry { data, dirty: false })
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
        use std::collections::hash_map::Entry;
        let entry = match self.cache.entry(sector) {
            Entry::Occupied(mut entry) => {
                entry.into_mut()
            }
            Entry::Vacant(entry) => {
                let mut data = Vec::with_capacity(self.device.sector_size() as usize);
                self.device.read_all_sector(sector, &mut data)?;
                entry.insert(CacheEntry { data, dirty: false })
            }
        };
        Ok(&entry.data)
    }
}

// FIXME: Implement `BlockDevice` for `CacheDevice`.
/*
impl BlockDevice for CacheDevice {
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        (*self).read_sector(n, buf)
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        (*self).write_sector(n, buf)
    }
}
*/


impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedDevice")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
