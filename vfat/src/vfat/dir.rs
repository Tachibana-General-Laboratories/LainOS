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
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    // FIXME: Fill me in.
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    // FIXME: Fill me in.
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
