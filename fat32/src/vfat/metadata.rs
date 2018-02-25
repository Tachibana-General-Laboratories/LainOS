use std::fmt;

use traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

impl Attributes {
    pub fn read_only(self) -> bool {
        (self.0 & 0x01) != 0
    }
    pub fn hidden(self) -> bool {
        (self.0 & 0x02) != 0
    }
    pub fn system(self) -> bool {
        (self.0 & 0x04) != 0
    }
    pub fn volume_id(self) -> bool {
        (self.0 & 0x08) != 0
    }
    pub fn directory(self) -> bool {
        (self.0 & 0x10) != 0
    }
    pub fn archive(self) -> bool {
        (self.0 & 0x20) != 0
    }
    pub fn lnf(self) -> bool {
        (self.0 & (0x01|0x02|0x04|0x08)) != 0
    }
}

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub attributes: Attributes,
    pub created: Timestamp,
    pub accessed: Timestamp,
    pub modified: Timestamp,
}

impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        1980 + (self.date.0 >> 8) as usize
    }
    fn month(&self) -> u8 {
        ((self.date.0 >> 4) & 0b111) as u8
    }
    fn day(&self) -> u8 {
        (self.date.0 & 0b1111) as u8
    }
    fn hour(&self) -> u8 {
        (self.time.0 >> 10) as u8
    }
    fn minute(&self) -> u8 {
        ((self.time.0 >> 4) & 0b11111) as u8
    }
    fn second(&self) -> u8 {
        2 * (self.time.0 & 0b1111) as u8
    }
}

// FIXME: Implement `traits::Metadata` for `Metadata`.
impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        self.attributes.read_only()
    }
    fn hidden(&self) -> bool {
        self.attributes.hidden()
    }
    fn created(&self) -> Self::Timestamp {
        self.created
    }
    fn accessed(&self) -> Self::Timestamp {
        self.accessed
    }
    fn modified(&self) -> Self::Timestamp {
        self.modified
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use traits::Timestamp;
        write!(f, "{}-{}-{} {}:{}:{}",
            self.year(), self.month(), self.day(),
            self.hour(), self.minute(), self.second(),
        )
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use traits::Metadata;
        writeln!(f, "ro: {}, hid: {}", self.read_only(), self.hidden())?;
        writeln!(f, "Access: {}", self.accessed())?;
        writeln!(f, "Modify: {}", self.modified())?;
        writeln!(f, "Create: {}", self.created())
    }
}
