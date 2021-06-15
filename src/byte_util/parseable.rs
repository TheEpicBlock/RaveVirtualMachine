use std::{fmt, error};
use std::io::{Cursor, Read};
use std::string::FromUtf8Error;

pub trait ByteParseable<ERR: error::Error> {
    fn parse_bytes(bytes: &[u8]) -> Result<Self, ERR> where Self: Sized {
        return Self::parse(&mut Cursor::new(bytes));
    }

    fn parse(bytes: &mut impl Read) -> Result<Self, ERR> where Self: Sized;

    fn parse_array(bytes: &mut impl Read, amount: usize) -> Result<Vec<Self>, ERR> where Self: Sized {
        let mut res = Vec::with_capacity(amount);
        for _ in 0..amount {
            res.push(Self::parse(bytes)?);
        }
        return Ok(res);
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use crate::byte_util::BigEndianReadExt;
    use crate::byte_util::parseable::{ByteParseable};

    #[derive(Eq, PartialEq, Debug)]
    struct Test(u8);

    impl ByteParseable<std::io::Error> for Test {
        fn parse(bytes: &mut impl Read) -> Result<Self, std::io::Error> where Self: Sized {
            Ok(Test(bytes.read_u8()?))
        }
    }

    #[test]
    fn vector_byte_parse() {
        let bytes = vec![1,2,3,5,8];
        let mut tests = Vec::with_capacity(bytes.len());
        for i in &bytes { tests.push(Test(*i)); }

        //Vector is now out original list of numbers. And byte_vector is the same but with the length appended at the front as a u16.
        assert_eq!(tests, Test::parse_array(&mut Cursor::new(bytes), tests.len()).unwrap())
    }
}