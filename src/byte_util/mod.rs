mod read_ext;
mod parseable;

pub use read_ext::BigEndianReadExt;
pub use parseable::ParseError;
pub use parseable::ByteParseable;
use std::io::Read;
use std::io;

/// Reads an amount of bytes to a vector.
pub fn read_to_vec(buffer: &mut impl Read, amount: usize) -> io::Result<Vec<u8>> {
    let mut vec = vec![0u8; amount];
    buffer.read_exact(&mut vec)?;

    Ok(vec)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::byte_util::{read_to_vec, BigEndianReadExt};

    #[test]
    fn test_read_to_vec() {
        let mut bytes = Cursor::new(&[0u8, 1, 2, 3, 4, 5, 6, 7]);
        let len = 4;
        let vec = read_to_vec(&mut bytes, 4).unwrap();

        assert_eq!(vec, vec![0, 1, 2, 3]);
        assert_eq!(bytes.read_u8().unwrap(), 4);
    }
}