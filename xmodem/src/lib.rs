use std::io;

#[cfg(test)] mod tests;
mod read_ext;
mod progress;

pub use progress::{Progress, ProgressFn};

use read_ext::ReadExt;

const SOH: u8 = 0x01;
const EOT: u8 = 0x04;
const ACK: u8 = 0x06;
const NAK: u8 = 0x15;
const CAN: u8 = 0x18;

/// Implementation of the XMODEM protocol.
pub struct Xmodem<R> {
    packet: u8,
    inner: R,
    started: bool,
    progress: ProgressFn
}

impl Xmodem<()> {
    /// Transmits `data` to the receiver `to` using the XMODEM protocol. If the
    /// length of the total data yielded by `data` is not a multiple of 128
    /// bytes, the data is padded with zeroes and sent to the receiver.
    ///
    /// Returns the number of bytes written to `to`, excluding padding zeroes.
    #[inline]
    pub fn transmit<R, W>(data: R, to: W) -> io::Result<usize>
        where W: io::Read + io::Write, R: io::Read
    {
        Self::transmit_with_progress(data, to, progress::noop)
    }

    /// Transmits `data` to the receiver `to` using the XMODEM protocol. If the
    /// length of the total data yielded by `data` is not a multiple of 128
    /// bytes, the data is padded with zeroes and sent to the receiver.
    ///
    /// The function `f` is used as a callback to indicate progress throughout
    /// the transmission. See the [`Progress`] enum for more information.
    ///
    /// Returns the number of bytes written to `to`, excluding padding zeroes.
    pub fn transmit_with_progress<R, W>(mut data: R, to: W, f: ProgressFn) -> io::Result<usize>
        where W: io::Read + io::Write, R: io::Read
    {
        let mut transmitter = Xmodem::new_with_progress(to, f);
        let mut packet = [0u8; 128];
        let mut written = 0;
        'next_packet: loop {
            let n = data.read_max(&mut packet)?;
            packet[n..].iter_mut().for_each(|b| *b = 0);

            if n == 0 {
                transmitter.write_packet(&[])?;
                return Ok(written);
            }

            for _ in 0..10 {
                match transmitter.write_packet(&packet) {
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                    Ok(_) => {
                        written += n;
                        continue 'next_packet;
                    }
                }
            }

            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "bad transmit"));
        }
    }

    /// Receives `data` from `from` using the XMODEM protocol and writes it into
    /// `into`. Returns the number of bytes read from `from`, a multiple of 128.
    #[inline]
    pub fn receive<R, W>(from: R, into: W) -> io::Result<usize>
       where R: io::Read + io::Write, W: io::Write
    {
        Self::receive_with_progress(from, into, progress::noop)
    }

    /// Receives `data` from `from` using the XMODEM protocol and writes it into
    /// `into`. Returns the number of bytes read from `from`, a multiple of 128.
    ///
    /// The function `f` is used as a callback to indicate progress throughout
    /// the reception. See the [`Progress`] enum for more information.
    pub fn receive_with_progress<R, W>(from: R, mut into: W, f: ProgressFn) -> io::Result<usize>
       where R: io::Read + io::Write, W: io::Write
    {
        let mut receiver = Xmodem::new_with_progress(from, f);
        let mut packet = [0u8; 128];
        let mut received = 0;
        'next_packet: loop {
            for _ in 0..10 {
                match receiver.read_packet(&mut packet) {
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                    Ok(0) => break 'next_packet,
                    Ok(n) => {
                        received += n;
                        into.write_all(&packet)?;
                        continue 'next_packet;
                    }
                }
            }

            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "bad receive"));
        }

        Ok(received)
    }
}

impl<T: io::Read + io::Write> Xmodem<T> {
    /// Returns a new `Xmodem` instance with the internal reader/writer set to
    /// `inner`. The returned instance can be used for both receiving
    /// (downloading) and sending (uploading).
    pub fn new(inner: T) -> Self {
        Self { packet: 1, started: false, inner, progress: progress::noop}
    }

    /// Returns a new `Xmodem` instance with the internal reader/writer set to
    /// `inner`. The returned instance can be used for both receiving
    /// (downloading) and sending (uploading). The function `f` is used as a
    /// callback to indicate progress throughout the transfer. See the
    /// [`Progress`] enum for more information.
    pub fn new_with_progress(inner: T, f: ProgressFn) -> Self {
        Self { packet: 1, started: false, inner, progress: f }
    }

    /// Reads a single byte from the inner I/O stream. If `abort_on_can` is
    /// `true`, an error of `ConnectionAborted` is returned if the read byte is
    /// `CAN`.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the inner stream fails or if
    /// `abort_on_can` is `true` and the read byte is `CAN`.
    fn read_byte(&mut self, abort_on_can: bool) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.inner.read_exact(&mut buf)?;

        let byte = buf[0];
        if abort_on_can && byte == CAN {
            return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "received CAN"));
        }

        Ok(byte)
    }

    /// Writes a single byte to the inner I/O stream.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the inner stream fails.
    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.inner.write_all(&[byte])
    }

    /// Reads a single byte from the inner I/O stream and compares it to `byte`.
    /// If the bytes match, the byte is returned as an `Ok`. If they differ and
    /// the read byte is not `CAN`, an error of `InvalidData` with the message
    /// `expected` is returned. If they differ and the read byte is `CAN`, an
    /// error of `ConnectionAborted` is returned. In either case, if they bytes
    /// differ, a `CAN` byte is written out to the inner stream.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the inner stream fails, if the read
    /// byte was not `byte`, if the read byte was `CAN` and `byte` is not `CAN`,
    /// or if writing the `CAN` byte failed on byte mismatch.
    fn expect_byte_or_cancel(&mut self, byte: u8, msg: &'static str) -> io::Result<u8> {
        let b = self.read_byte(false)?;
        if b == byte {
            Ok(byte)
        } else {
            self.write_byte(CAN)?;
            if b == CAN {
                Err(io::Error::new(io::ErrorKind::ConnectionAborted, msg))
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidData, msg))
            }
        }
    }

    /// Reads a single byte from the inner I/O stream and compares it to `byte`.
    /// If they differ, an error of `InvalidData` with the message `expected` is
    /// returned. Otherwise the byte is returned. If `byte` is not `CAN` and the
    /// read byte is `CAN`, a `ConnectionAborted` error is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the inner stream fails, or if the read
    /// byte was not `byte`. If the read byte differed and was `CAN`, an error
    /// of `ConnectionAborted` is returned. Otherwise, the error kind is
    /// `InvalidData`.
    fn expect_byte(&mut self, byte: u8, expected: &'static str) -> io::Result<u8> {
        let b = self.read_byte(byte != CAN)?;
        if b == byte {
            Ok(b)
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, expected))
        }
    }

    /// Reads (downloads) a single packet from the inner stream using the XMODEM
    /// protocol. On success, returns the number of bytes read (always 128).
    ///
    /// The progress callback is called with `Progress::Start` when reception
    /// for the first packet has started and subsequently with
    /// `Progress::Packet` when a packet is received successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or writing to the inner stream fails at any
    /// point. Also returns an error if the XMODEM protocol indicates an error.
    /// In particular, an `InvalidData` error is returned when:
    ///
    ///   * The sender's first byte for a packet isn't `EOT` or `SOH`.
    ///   * The sender doesn't send a second `EOT` after the first.
    ///   * The received packet numbers don't match the expected values.
    ///
    /// An error of kind `Interrupted` is returned if a packet checksum fails.
    ///
    /// An error of kind `ConnectionAborted` is returned if a `CAN` byte is
    /// received when not expected.
    ///
    /// An error of kind `UnexpectedEof` is returned if `buf.len() < 128`.
    pub fn read_packet(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 128 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "buf.len() < 128"))
        }

        if !self.started {
            self.write_byte(NAK)?;
            (self.progress)(Progress::Started);
            self.started = true;
        }

        // Waits for an SOH or EOT byte from the sender.
        match self.read_byte(true)? {
            SOH => {
                let p = self.packet;
                let c = 0xFFu8.wrapping_sub(p);
                // Reads the next byte and compares it to the current packet number.
                self.expect_byte_or_cancel(p, "packet")?;
                // Reads the next byte and compares it to the 1s complement of the packet number.
                self.expect_byte_or_cancel(c, "complement")?;

                // Reads a packet (128 bytes) from the sender.
                let mut sum = 0u8;
                for b in buf.iter_mut() {
                    *b = self.read_byte(false)?;
                    sum = sum.wrapping_add(*b);
                }

                // Reads the next byte and compares it to the computed checksum.
                if self.read_byte(false)? != sum {
                    self.write_byte(NAK)?;
                    Err(io::Error::new(io::ErrorKind::Interrupted, "fail checksum"))
                } else {
                    self.write_byte(ACK)?;
                    (self.progress)(Progress::Packet(self.packet));
                    self.packet = self.packet.wrapping_add(1);
                    Ok(128)
                }
            },
            EOT => {
                self.started = false;
                self.write_byte(NAK)?;
                self.expect_byte_or_cancel(EOT, "end of transmission")?;
                self.write_byte(ACK)?;
                Ok(0)
            },

            _ => {
                self.write_byte(CAN)?;
                Err(io::Error::new(io::ErrorKind::InvalidData, "wtf?"))
            },
        }
    }

    /// Sends (uploads) a single packet to the inner stream using the XMODEM
    /// protocol. If `buf` is empty, end of transmissions is sent. Users of this
    /// interface should ensure that `write_packet(&[])` is called when data
    /// transmission is complete. On success, returns the number of bytes
    /// written.
    ///
    /// The progress callback is called with `Progress::Waiting` before waiting
    /// for the receiver's `NAK`, `Progress::Start` when transmission of the
    /// first packet has started and subsequently with `Progress::Packet` when a
    /// packet is sent successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or writing to the inner stream fails at any
    /// point. Also returns an error if the XMODEM protocol indicates an error.
    /// In particular, an `InvalidData` error is returned when:
    ///
    ///   * The receiver's first byte isn't a `NAK`.
    ///   * The receiver doesn't respond with a `NAK` to the first `EOT`.
    ///   * The receiver doesn't respond with an `ACK` to the second `EOT`.
    ///   * The receiver responds to a complete packet with something besides
    ///     `ACK` or `NAK`.
    ///
    /// An error of kind `UnexpectedEof` is returned if `buf.len() < 128 &&
    /// buf.len() != 0`.
    ///
    /// An error of kind `ConnectionAborted` is returned if a `CAN` byte is
    /// received when not expected.
    ///
    /// An error of kind `Interrupted` is returned if a packet checksum fails.
    pub fn write_packet(&mut self, buf: &[u8]) -> io::Result<usize> {
        (self.progress)(Progress::Waiting);
        if !self.started {
            self.expect_byte(NAK, "wait NAK")?;
            self.started = true;
            (self.progress)(Progress::Started);
        }

        if buf.len() == 0 {
            self.started = false;
            self.write_byte(EOT)?;
            self.expect_byte(NAK, "wait NAK")?;
            self.write_byte(EOT)?;
            self.expect_byte(ACK, "wait ACK")?;
            Ok(0)
        } else if buf.len() < 128 {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "buf.len() < 128"))
        } else {
            self.write_byte(SOH)?;

            let num = self.packet;
            self.write_byte(num)?;
            let num = 0xFFu8.wrapping_sub(num);
            self.write_byte(num)?;

            let mut sum = 0u8;
            for &b in buf {
                self.write_byte(b)?;
                sum = sum.wrapping_add(b);
            }
            self.write_byte(sum)?;

            match self.read_byte(false)? {
                ACK => {
                    (self.progress)(Progress::Packet(self.packet));
                    self.packet = self.packet.wrapping_add(1);
                    Ok(128)
                }
                NAK => Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted")),
                _ =>   Err(io::Error::new(io::ErrorKind::InvalidData, "end")),
            }

        }
    }

    /// Flush this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    ///
    /// # Errors
    ///
    /// It is considered an error if not all bytes could be written due to I/O
    /// errors or EOF being reached.
    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
