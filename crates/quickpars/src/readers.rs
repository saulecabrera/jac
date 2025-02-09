//! Bytecode reader.

use anyhow::{ensure, Result};
use std::io::Cursor;

/// A general binary reader.
#[derive(Debug, Copy, Clone)]
pub struct BinaryReader<'a> {
    /// A reference to the data that the reader operates on.
    data: &'a [u8],
    /// The offset of the binary reader.
    pub offset: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn empty() -> Self {
        Self {
            data: Default::default(),
            offset: 0,
        }
    }

    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Returns a reference to the underlying data.
    pub(crate) fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn done(&self) -> bool {
        self.offset >= self.data.len()
    }

    /// Reads the requested amount of bytes, returning a slice of the bytes.
    fn read(&mut self, bytes: usize) -> Result<&'a [u8]> {
        self.ensure(bytes).map(|_| {
            let start = self.offset;
            self.offset += bytes;
            &self.data[start..self.offset]
        })
    }

    /// Reads a single byte.
    pub fn read_u8(&mut self) -> Result<u8> {
        let slice = self.read(1)?;
        // TODO: Get rid of the `expect`.
        let byte = slice.first().expect("single byte to be available");

        Ok(*byte)
    }

    /// Reads two bytes into a `u16`.
    pub fn read_u16(&mut self) -> Result<u16> {
        let slice = self.read(2)?;
        Ok(u16::from_le_bytes(slice.try_into()?))
    }

    /// Reads four bytes into a `u32`.
    pub fn read_u32(&mut self) -> Result<u32> {
        let slice = self.read(4)?;
        Ok(u32::from_le_bytes(slice.try_into()?))
    }

    /// Reads 8 bytes into a `u64`.
    pub fn read_u64(&mut self) -> Result<u64> {
        let slice = self.read(8)?;
        Ok(u64::from_le_bytes(slice.try_into()?))
    }

    /// Reads a single atom.
    pub fn read_atom(&mut self) -> Result<u32> {
        Ok(self.read_leb128()? >> 1)
    }

    /// Reads an integer in LEB-128 format.
    pub fn read_leb128(&mut self) -> Result<u32> {
        let mut cursor = Cursor::new(&self.data[self.offset..]);
        let val = leb128::read::unsigned(&mut cursor)?;
        let bytes_read = cursor.position();

        self.offset += bytes_read as usize;

        Ok(u32::try_from(val)?)
    }

    /// Reads a signed integer in LEB-128 format.
    pub fn read_sleb128(&mut self) -> Result<i32> {
        let mut cursor = Cursor::new(&self.data[self.offset..]);
        let val = leb128::read::signed(&mut cursor)?;
        let bytes_read = cursor.position();

        self.offset += bytes_read as usize;

        Ok(i32::try_from(val)?)
    }

    /// Skips the specified number of bytes.
    pub fn skip(&mut self, bytes: usize) -> Result<()> {
        self.ensure(bytes).map(|_| {
            self.offset += bytes;
            ()
        })
    }

    /// Validates that the underlying data has at least `size` bytes.
    fn ensure(&self, size: usize) -> Result<()> {
        let req = self.offset + size;
        ensure!(
            req <= self.data.len(),
            "Tried to read more bytes than available"
        );

        Ok(())
    }
}

/// Creates a [BinaryReader] slice for a bytecode section.
pub(crate) fn slice<'a>(reader: &mut BinaryReader<'a>, size: usize) -> Result<BinaryReader<'a>> {
    let data = reader.data();
    let slice = &data[reader.offset..(reader.offset + size)];
    let res = BinaryReader::new(slice);
    reader.skip(size)?;
    Ok(res)
}

/// Reads the bytes representing a QuickJS string.
pub(crate) fn read_str_bytes<'a>(reader: &mut BinaryReader<'a>) -> Result<&'a [u8]> {
    let mut len = reader.read_leb128()?;
    // The last bit of the length encodes if the atom is a wide char.
    let is_wide_char = len & 1;
    // Once we have read the `wide_char` bit, we clear it out.
    len >>= 1;
    let size = (len << is_wide_char) as usize;
    let res = &reader.data()[reader.offset..(reader.offset + size)];
    reader.skip(size)?;

    Ok(res)
}
