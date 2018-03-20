use std::ffi::OsStr;
use std::char::decode_utf16;
use std::borrow::Cow;
use std::io;

use traits;
use util::VecExt;
use vfat::{VFat, Shared, File, Cluster, Entry};
use vfat::{Metadata, Attributes, Timestamp, Time, Date};

#[derive(Debug)]
pub struct Dir {
    pub name: String,
    pub meta: Metadata,
    pub vfat: Shared<VFat>,
    pub cluster: Cluster,
    pub size: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatRegularDirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attributes: Attributes,
    _reserved_nt: u8,
    _creat: u8,
    create_time: Time,
    create_date: Date,
    last_access_date: Date,
    hi_cluster: u16,
    mod_time: Time,
    mod_date: Date,
    lo_cluster: u16,
    size: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    seq: u8,
    name0: [u16; 5],
    attributes: Attributes,
    _type: u8,
    checksum: u8,
    name1: [u16; 6],
    _zero: u16,
    name2: [u16; 2],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    pub stub: [u8; 32],
}

#[derive(Copy, Clone)]
pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl VFatDirEntry {
    fn and_end(self) -> Option<Self> {
        if unsafe { self.unknown.stub[0] != 0 } {
            Some(self)
        } else {
            None
        }
    }

    fn is_end(&self) -> bool {
        unsafe { self.unknown.stub[0] == 0 }
    }
    fn is_unused(&self) -> bool {
        unsafe { self.unknown.stub[0] == 0xE5 }
    }
    fn is_long(&self) -> bool {
        unsafe { self.unknown.stub[11] == 0x0F }
    }

    fn cluster(&self) -> Cluster {
        unsafe {
            let lo = self.regular.lo_cluster as u32;
            let hi = self.regular.hi_cluster as u32;
            Cluster::from(lo | hi << 16)
        }
    }

    fn size(&self) -> u64 {
        unsafe { self.regular.size as u64 }
    }

    fn meta(&self) -> Metadata {
        let regular = unsafe { &self.regular };
        let attributes = regular.attributes;
        let created = Timestamp {
            time: regular.create_time,
            date: regular.create_date,
        };
        let modified = Timestamp {
            time: regular.mod_time,
            date: regular.mod_date,
        };
        let accessed = Timestamp {
            time: Time(0),
            date: regular.last_access_date,
        };

        Metadata {
            attributes,
            created,
            accessed,
            modified,
        }
    }

    fn long_name(&self) -> String {
        use std::char::{decode_utf16, REPLACEMENT_CHARACTER};

        unsafe {
            let name0 = (&self.long_filename.name0 as *const [u16; 5]).read_unaligned();
            let name1 = (&self.long_filename.name1 as *const [u16; 6]).read_unaligned();
            let name2 = (&self.long_filename.name2 as *const [u16; 2]).read_unaligned();

            let name0 = name0.iter().cloned();
            let name1 = name1.iter().cloned();
            let name2 = name2.iter().cloned();

            let name0 = decode_utf16(name0);
            let name1 = decode_utf16(name1);
            let name2 = decode_utf16(name2);

            name0.chain(name1).chain(name2)
                .take_while(|r| r.is_ok())
                .map(|r| r.unwrap_or('#'))
                .take_while(|&r| r != '\u{0}' && r != '\u{ffff}')
                .collect()
        }
    }

    fn short_name(&self) -> String {
        let name = unsafe { &self.regular.name[..] };
        let ext = unsafe { &self.regular.ext[..] };

        let mut name: String = name.iter()
            .filter(|&&c| c != 0 && c != b' ')
            .map(|&c| c as char)
            .collect();

        let ext: String = ext.iter()
            .filter(|&&c| c != 0 && c != b' ')
            .map(|&c| c as char)
            .collect();

        if ext.len() > 0 {
            name + "." + &ext
        } else {
            name
        }
    }
}

impl Dir {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
        use traits::*;

        if let Some(name) = name.as_ref().to_str() {
            let entry = self.entries()?
                .find(|e| e.name().eq_ignore_ascii_case(name));
            if let Some(entry) = entry {
                Ok(entry)
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, name.to_string()))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "fail name"))
        }
    }
    pub fn root(vfat: Shared<VFat>) -> Self {
        let cluster = vfat.borrow().root_dir_cluster;
        Self {
            name: "".to_string(),
            meta: unsafe { ::std::mem::zeroed() },
            vfat,
            cluster,
            size: 0,
        }
    }
}

impl traits::Dir for Dir {
    type Entry = Entry;
    type Iter = DirIter;

    fn entries(&self) -> io::Result<Self::Iter> {
        let entries = {
            let mut entries = Vec::new();
            let mut vfat = self.vfat.borrow_mut();
            vfat.read_chain(self.cluster, &mut entries)?;
            entries
        };
        Ok(DirIter {
            entries,
            vfat: self.vfat.clone(),
            name: String::new(),
            current: 0,
        })
    }
}

pub struct DirIter {
    entries: Vec<u8>,
    vfat: Shared<VFat>,
    current: usize,
    name: String,
}

impl DirIter {
    fn next_dir_entry(&mut self) -> Option<VFatDirEntry> {
        if self.current < self.entries.len()  {
            Some(unsafe {
                let p = self.entries.as_ptr() as *const VFatDirEntry;
                p.offset(self.current as isize).read_unaligned()
            })
        } else {
            None
        }
    }
}

impl Iterator for DirIter {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let e = self.next_dir_entry().and_then(VFatDirEntry::and_end)?;
            self.current += 1;

            if e.is_unused() {
                continue;
            }

            if e.is_long()  {
                let s: String = e.long_name();
                if s.len() >= 3 && &s[..3] == "._." {
                    continue;
                }
                self.name = s + &self.name;
                continue;
            }

            if self.name.is_empty() {
                self.name += &e.short_name();
            }

            let name = self.name.clone();
            self.name.clear();

            let meta = e.meta();
            let e = if meta.attributes.directory() {
                Entry::Dir(Dir {
                    name,
                    meta,
                    vfat: self.vfat.clone(),
                    size: e.size(),
                    cluster: e.cluster(),
                })
            } else {
                Entry::File(File {
                    name,
                    meta,
                    vfat: self.vfat.clone(),
                    size: e.size(),
                    cluster: e.cluster(),
                    position: 0,
                })
            };
            return Some(e);
        }
    }
}
