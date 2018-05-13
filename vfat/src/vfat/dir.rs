use std::{io, mem};
use std::ffi::OsStr;
use std::char::decode_utf16;
use std::borrow::Cow;

use std::string::{String, ToString};
use std::vec::Vec;

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
    fn and_end(&self) -> Option<Self> {
        if !self.is_end() {
            Some(*self)
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
        let lfn: ([u16; 5], [u16; 6], [u16; 2]) = unsafe {
            let lfn = &self.long_filename;
            (
                (&lfn.name0 as *const [u16; 5]).read_unaligned(),
                (&lfn.name1 as *const [u16; 6]).read_unaligned(),
                (&lfn.name2 as *const [u16; 2]).read_unaligned(),
            )
        };

        let mut name = [0u16; 5+6+2];
        name[0  ..5].copy_from_slice(&lfn.0[..]);
        name[5  ..5+6].copy_from_slice(&lfn.1[..]);
        name[5+6..5+6+2].copy_from_slice(&lfn.2[..]);

        let iter = decode_utf16(name.iter().cloned())
            .take_while(|r| r.is_ok())
            .map(|r| r.unwrap_or('#'))
            .take_while(|&r| r != '\u{0}' && r != '\u{ffff}');
        let mut s = String::with_capacity(13);
        s.extend(iter);
        s
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

        let name = name.as_ref().to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "`name` contains invalid UTF-8 characters"))?;

        self.entries()?
            .find(|e| e.name().eq_ignore_ascii_case(name))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file notfound"))
    }
    pub fn root(vfat: Shared<VFat>) -> Self {
        let cluster = vfat.borrow().root_dir_cluster;
        Self {
            name: "".to_string(),
            meta: unsafe { mem::zeroed() },
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
        let entries: Vec<VFatDirEntry> = {
            let mut entries = Vec::new();
            let mut vfat = self.vfat.borrow_mut();
            vfat.read_chain(self.cluster, &mut entries)?;
            unsafe { entries.cast() }
        };
        Ok(DirIter {
            entries,
            vfat: self.vfat.clone(),
            name: String::with_capacity(64),
            current: 0,
        })
    }
}

pub struct DirIter {
    entries: Vec<VFatDirEntry>,
    vfat: Shared<VFat>,
    current: usize,
    name: String,
}

impl Iterator for DirIter {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        for e in self.entries.iter().skip(self.current) {
            //.and_then(VFatDirEntry::and_end)?;
            self.current += 1;

            if e.is_end() {
                continue;
            }

            if e.is_unused() {
                continue;
            }

            if e.is_long() {
                let s = e.long_name();
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

        None
    }
}
