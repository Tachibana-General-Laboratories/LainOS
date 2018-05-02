pub trait Read {
    /// Reads a byte from the device, blocking until a byte is available.
    pub fn read_byte(&mut self) -> Result<u8>;
}

pub trait Write {
    /// Writes the byte `byte` to the device.
    pub fn write_byte(&mut self, byte: u8) -> Result<()>;
}

pub trait CharDevice: Read + Write {}

pub fn pipe_copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64>
    where R: Read, W: Write,
{
    loop {
        writer.write(reader.read()?)?;
    }
}
