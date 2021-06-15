use std::io;
use std::io::{Cursor, Read, Result};
use std::prelude::v1;

use byteorder::BigEndian;
use byteorder::ReadBytesExt;

/// Extends [`Read`] with methods for reading numbers. (For `std::io`.)
///
/// All of these methods are explicitly big endian
///
/// # Examples
///
/// Read unsigned 16 bit big-endian integers from a [`Read`]:
///
/// ```
/// use std::io::Cursor;
/// use byteutil::BigEndianReadExt
///
/// let mut rdr = Cursor::new(vec![2, 5, 3, 0]);
/// assert_eq!(517, rdr.read_u16().unwrap());
/// assert_eq!(768, rdr.read_u16().unwrap());
/// ```
///
/// [`BigEndian`]: enum.BigEndian.html
/// [`LittleEndian`]: enum.LittleEndian.html
/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
pub trait BigEndianReadExt: io::Read {
    /// Reads an unsigned 8 bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Reads an unsigned 16 bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u16(&mut self) -> Result<u16> {
        self::ReadBytesExt::read_u16::<BigEndian>(self)
    }

    /// Reads an unsigned 32 bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u32(&mut self) -> Result<u32> {
        self::ReadBytesExt::read_u32::<BigEndian>(self)
    }

    /// Reads an unsigned 64 bit integer from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_u64(&mut self) -> Result<u64> {
        self::ReadBytesExt::read_u64::<BigEndian>(self)
    }

    /// Reads a 32 bit float from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_f32(&mut self) -> Result<f32> {
        self::ReadBytesExt::read_f32::<BigEndian>(self)
    }

    /// Reads a 64 bit float from the underlying reader.
    ///
    /// # Errors
    ///
    /// This method returns the same errors as [`Read::read_exact`].
    ///
    /// [`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact
    #[inline]
    fn read_f64(&mut self) -> Result<f64> {
        self::ReadBytesExt::read_f64::<BigEndian>(self)
    }
}

impl<R: io::Read + ?Sized> BigEndianReadExt for R {}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use super::BigEndianReadExt;

    #[test]
    fn read_u8() {
        assert_eq!(testing_cursor().read_u8().unwrap(), 0x01)
    }

    #[test]
    fn read_u16() {
        assert_eq!(testing_cursor().read_u16().unwrap(), 0x0102)
    }

    #[test]
    fn read_u32() {
        assert_eq!(testing_cursor().read_u32().unwrap(), 0x01020304)
    }

    fn testing_cursor() -> Cursor<&'static [u8]> {
        Cursor::new(&[0x01, 0x02, 0x03, 0x04])
    }
}
