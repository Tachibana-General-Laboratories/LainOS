use alloc::string::{String, ToString};
use core::result;

pub type Result<T> = result::Result<T, Error>;

/// A list specifying general categories of I/O error.
///
/// This list is intended to grow over time and it is not recommended to
/// exhaustively match against it.
///
/// It is used with the [`io::Error`] type.
///
/// [`io::Error`]: struct.Error.html
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(deprecated)]
pub enum Error {
    /// An entity was not found, often a file.
    NotFound(String),
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied(String),
    /// The connection was refused by the remote server.
    ConnectionRefused(String),
    /// The connection was reset by the remote server.
    ConnectionReset(String),
    /// The connection was aborted (terminated) by the remote server.
    ConnectionAborted(String),
    /// The network operation failed because it was not connected yet.
    NotConnected(String),
    /// A socket address could not be bound because the address is already in
    /// use elsewhere.
    AddrInUse(String),
    /// A nonexistent interface was requested or the requested address was not
    /// local.
    AddrNotAvailable(String),
    /// The operation failed because a pipe was closed.
    BrokenPipe(String),
    /// An entity already exists, often a file.
    AlreadyExists(String),
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    WouldBlock(String),
    /// A parameter was incorrect.
    InvalidInput(String),
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: #variant.InvalidInput
    InvalidData(String),
    /// The I/O operation's timeout expired, causing it to be canceled.
    TimedOut(String),
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    ///
    /// This typically means that an operation could only succeed if it wrote a
    /// particular number of bytes but only a smaller number of bytes could be
    /// written.
    ///
    /// [`write`]: ../../std/io/trait.Write.html#tymethod.write
    /// [`Ok(0)`]: ../../std/io/type.Result.html
    WriteZero(String),
    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    Interrupted(String),
    /// Any I/O error not part of this list.
    Other(String),

    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    UnexpectedEof(String),

    /*
    /// A marker variant that tells the compiler that users of this enum cannot
    /// match it exhaustively.
    #[unstable(feature = "io_error_internals",
               reason = "better expressed through extensible enums that this \
                         enum cannot be exhaustively matched against",
               issue = "0")]
    #[doc(hidden)]
    __Nonexhaustive,
    */
}

pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; }
                Err(Error::Interrupted(_)) => {}
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(Error::UnexpectedEof("failed to fill whole buffer".to_string()))
        } else {
            Ok(())
        }
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

use core::{mem, cmp};

impl<'a> Write for &'a mut [u8] {
    #[inline]
    fn write(&mut self, data: &[u8]) -> Result<usize> {
        let amt = cmp::min(data.len(), self.len());
        let (a, b) = mem::replace(self, &mut []).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(amt)
    }

    /*
    #[inline]
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        if self.write(data)? == data.len() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::WriteZero, "failed to write whole buffer"))
        }
    }
    */

    #[inline]
    fn flush(&mut self) -> Result<()> { Ok(()) }
}
