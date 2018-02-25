use std::cmp::{min, max};
use std::io::{self, SeekFrom};

use traits;
use vfat::{VFat, Shared, Cluster, Metadata};

#[derive(Debug)]
pub struct File {
    // FIXME: Fill me in.
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
        unimplemented!("File::seek()")
    }
}
