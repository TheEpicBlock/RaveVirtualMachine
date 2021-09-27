mod parseable;
mod read_ext;

pub use parseable::ByteParseable;
pub use read_ext::BigEndianReadExt;
use std::io;
use std::io::Read;
use crate::ClassParseError;

/// Reads an amount of bytes to a vector.
pub fn read_to_vec(buffer: &mut impl Read, amount: usize) -> io::Result<Vec<u8>> {
    let mut vec = vec![0u8; amount];
    buffer.read_exact(&mut vec)?;

    Ok(vec)
}

pub fn parse_multiple<T: ByteParseable>(bytes: &mut impl Read, amount: usize) -> Result<Vec<T>, ClassParseError> {
    let mut result = Vec::with_capacity(amount);
    for _ in 0..amount {
        result.push(T::parse(bytes)?);
    }
    return Ok(result);
}

#[cfg(test)]
mod tests {
    use crate::byte_util::{read_to_vec, BigEndianReadExt, parse_multiple};
    use std::io::Cursor;
    use std::process::exit;

    #[test]
    fn test_read_to_vec() {
        let mut bytes = Cursor::new(&[0u8, 1, 2, 3, 4, 5, 6, 7]);
        let len = 4;
        let vec = read_to_vec(&mut bytes, 4).unwrap();

        assert_eq!(vec, vec![0, 1, 2, 3]);
        assert_eq!(bytes.read_u8().unwrap(), 4);
    }

    #[test]
    fn test_parse_multiple() {
        let bytes: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04];
        let expected = vec![0x0102, 0x0304];

        // Parses the bytes as u16's
        let parsed: Vec<u16> = parse_multiple(&mut Cursor::new(bytes), expected.len()).unwrap();
        assert_eq!(parsed, expected)
    }
}
