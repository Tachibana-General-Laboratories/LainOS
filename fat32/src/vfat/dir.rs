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
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    // FIXME: Fill me in.
    name: [u8; 8],
    ext: [u8; 3],
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
    name: [u8; 10],
    attributes: Attributes,
    _type: u8,
    checksum: u8,
    sname: [u8; 12],
    _zero: u16,
    tname: [u8; 4],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    // FIXME: Fill me in.
    stub: [u8; 32],
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
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
