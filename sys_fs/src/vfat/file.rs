use std::cmp::{min, max};
use std::io::{self, SeekFrom};

use traits;
use vfat::{VFat, Shared, Cluster, Metadata};

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub meta: Metadata,
    pub vfat: Shared<VFat>,
    pub cluster: Cluster,
    pub size: u64,

    pub position: u64,
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.

impl io::Seek for File {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (base, offset) = match pos {
            SeekFrom::Start(pos) => (pos, 0),
            SeekFrom::End(pos) => (self.size, pos),
            SeekFrom::Current(pos) => (self.position, pos),
        };
        let pos = if offset >= 0 {
            base.checked_add(offset as u64)
        } else {
            base.checked_sub((offset.wrapping_neg()) as u64)
        };
        match pos {
            Some(pos) if pos <= self.size => {
                self.position = pos;
                Ok(pos)
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid seek")),
        }
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.size {
            Ok(0)
        } else {
            let mut end = ((buf.len() as u64).min(self.size - self.position)) as usize;
            let mut vfat = self.vfat.borrow_mut();
            let n = vfat.read_cluster(self.cluster, self.position as usize, &mut buf[..end])?;
            self.position += n as u64;
            Ok(n)
        }
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }
    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}

impl traits::File for File {
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn size(&self) -> u64 {
        self.size
    }
}
