use std::io;
use sys::fs::BlockDevice;

use pi::common::spin_sleep_us;

extern "C" {
    /// A global representing the last SD controller error that occured.
    static sd_err: i64;

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occured, or -2 if an error sending commands to
    /// the SD controller occured.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occured or -2 if an
    /// error sending commands to the SD controller occured. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}

// for use by `libsd`.
#[no_mangle]
pub extern "C" fn wait_micros(n: u32) {
    spin_sleep_us(n)
}

#[derive(Debug)]
pub enum Error {
    Timeout,
    Sending,
    Other(i32),
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    pub fn new() -> Result<Self, Error> {
        match unsafe { sd_init() } {
            0 => Ok(Sd),
            -1 => Err(Error::Timeout),
            -2 => Err(Error::Sending),
            err @ _ => Err(Error::Other(err)),
        }
    }
}

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 512 || buf.len() > ::std::i32::MAX as usize {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "buf.len() out of bound"))
        } else {
            let n = unsafe { sd_readsector(n as i32, buf.as_mut_ptr()) };
            let errno = unsafe { sd_err };
            if n > 0 {
                Ok(n as usize)
            } else if errno == -1 {
                Err(io::Error::new(io::ErrorKind::TimedOut, "sd card timeout"))
            } else {
                Err(io::Error::new(io::ErrorKind::Other, format!("sd_err: {}", errno)))
            }
        }
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
