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
    // FIXME: Fill me in.
    name: String,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatRegularDirEntry {
    // FIXME: Fill me in.
    pub name: [u8; 8],
    pub ext: [u8; 3],
    attributes: Attributes,
    _reserved_nt: u8,
    _creat: u8,
    create_time: Time,
    create_date: Date,
    last_access_date: u16,
    access_rights: u16,
    mod_time: Time,
    mod_date: Date,
    start: u16,
    size: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    seq: u8,
    pub name0: [u16; 5],
    attributes: Attributes,
    _type: u8,
    checksum: u8,
    pub name1: [u16; 6],
    _zero: u16,
    pub name2: [u16; 2],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    pub stub: [u8; 32],
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    pub regular: VFatRegularDirEntry,
    pub long_filename: VFatLfnDirEntry,
}

impl VFatDirEntry {
    fn and_end(&self) -> Option<&Self> {
        if unsafe { self.unknown.stub[0] == 0 } {
            Some(self)
        } else {
            None
        }
    }
    pub fn is_end(&self) -> bool {
        unsafe { self.unknown.stub[0] == 0 }
    }
    pub fn is_unused(&self) -> bool {
        unsafe { self.unknown.stub[0] == 0xE5 }
    }
    pub fn is_long(&self) -> bool {
        unsafe { self.unknown.stub[11] == 0x0F }
    }
    pub fn long_name<'a>(&'a self) -> impl Iterator<Item=char> + 'a {
        unsafe {
        use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
        let name0 = self.long_filename.name0.iter().cloned();
        let name1 = self.long_filename.name1.iter().cloned();
        let name2 = self.long_filename.name2.iter().cloned();
        decode_utf16(name0)
            .chain(decode_utf16(name1))
            .chain(decode_utf16(name2))
            .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        }
    }
    pub fn short_name(&self) -> Result<&str, ::std::str::Utf8Error> {
        use std::str::from_utf8;
        let filter: &[_] = &['\0', ' '];
        let s = unsafe { &self.regular.name[..] };
        Ok(from_utf8(s)?.trim_right_matches(filter))
    }

    pub fn short_ext(&self) -> Result<&str, ::std::str::Utf8Error> {
        use std::str::from_utf8;
        let filter: &[_] = &['\0', ' '];
        let s = unsafe { &self.regular.ext[..] };
        Ok(from_utf8(s)?.trim_right_matches(filter))
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
        unimplemented!("Dir::find()")
    }
}

// FIXME: Implement `trait::Dir` for `Dir`.

pub struct DirIter<'a> {
    entries: ::std::slice::Iter<'a, VFatDirEntry>,
    name: String,
}

impl<'a> DirIter<'a> {
    pub fn new(entries: &'a [VFatDirEntry]) -> Self {
        Self {
            entries: entries.into_iter(),
            name:  String::with_capacity(256),
        }
    }
}

impl<'a> Iterator for DirIter<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let e = self.entries.next()
                .and_then(VFatDirEntry::and_end)?;
            if e.is_unused() {
                continue;
            }
            if e.is_long() {
                self.name.extend(e.long_name());
                continue;
            }

            if self.name.is_empty() {
                self.name.extend(e.short_name().unwrap().chars());
                self.name.push('.');
                self.name.extend(e.short_ext().unwrap().chars());
            }

            let name = self.name.clone();
            self.name.clear();
            return Some(name);
        }
    }
}
