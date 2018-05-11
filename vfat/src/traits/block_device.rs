use std::io;

/// Trait implemented by devices that can be read/written in sector
/// granularities.
pub trait BlockDevice: Send {
    /// Sector size in bytes. Must be a multiple of 512 >= 512. Defaults to 512.
    fn sector_size(&self) -> u64 {
        512
    }

    /// Read sector number `n` into `buf`.
    ///
    /// `self.sector_size()` or `buf.len()` bytes, whichever is less, are read
    /// into `buf`. The number of bytes read is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if seeking or reading from `self` fails.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize>;

    /// Append sector number `n` into `vec`.
    ///
    /// `self.sector_size()` bytes are appended to `vec`. The number of bytes
    /// read is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if seeking or reading from `self` fails.
    fn read_all_sector(&mut self, n: u64, vec: &mut Vec<u8>) -> io::Result<usize> {
        let sector_size = self.sector_size() as usize;

        let start = vec.len();
        let available = vec.capacity() - start;
        if available < sector_size {
            vec.reserve(sector_size - available);
        }

        unsafe { vec.set_len(start + sector_size); }
        let read = self.read_sector(n, &mut vec[start..])?;
        unsafe { vec.set_len(start + read); }
        Ok(read)
    }

    /// Overwrites sector `n` with the contents of `buf`.
    ///
    /// `self.sector_size()` or `buf.len()` bytes, whichever is less, are written
    /// to the sector. The number of byte written is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if seeking or writing to `self` fails. Returns an
    /// error of `UnexpectedEof` if the length of `buf` is less than
    /// `self.sector_size()`.
    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize>;
}

impl<'a, T: BlockDevice> BlockDevice for &'a mut T {
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        (*self).read_sector(n, buf)
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        (*self).write_sector(n, buf)
    }
}

macro impl_for_read_write_seek($(<$($gen:tt),*>)* $T:path) {
    use std::io::{Read, Write, Seek};

    impl $(<$($gen),*>)* BlockDevice for $T {
        fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
            let sector_size = self.sector_size();
            let to_read = ::std::cmp::min(sector_size as usize, buf.len());
            self.seek(io::SeekFrom::Start(n * sector_size))?;
            self.read_exact(&mut buf[..to_read])?;
            Ok(to_read)
        }

        fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
            let sector_size = self.sector_size();
            let to_write = ::std::cmp::min(sector_size as usize, buf.len());
            self.seek(io::SeekFrom::Start(n * sector_size))?;
            self.write_all(&buf[..to_write])?;
            Ok(to_write)
        }
    }
}

impl_for_read_write_seek!(<'a> ::std::io::Cursor<&'a mut [u8]>);
impl_for_read_write_seek!(::std::io::Cursor<Vec<u8>>);
impl_for_read_write_seek!(::std::io::Cursor<Box<[u8]>>);
#[cfg(test)] impl_for_read_write_seek!(::std::fs::File);
