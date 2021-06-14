use core::fmt;
use std::error::Error;
use std::io::{Cursor, Read};
use crate::byte_util::BigEndianReadExt;
use std::iter::Zip;
use std::string::FromUtf8Error;
use crate::class_file::parsing::ParsedClass;

mod parsing;
mod constantpool;

pub fn parse(bytes: &mut impl Read) -> Result<ParsedClass, ParseError> {
    ParsedClass::parse(bytes)
}

/// A class goes through multiple stages before being used. This enum keeps track of them
pub enum Stage {
    Parsing
}

#[derive(Debug)]
pub enum ParseError {
    /// The wrong magic value was found. The value here is what was found instead
    WrongMagic(u32),
    IoError(std::io::Error),
    InvalidConstantTableEntry(u8),
    Utf8Error(FromUtf8Error)
}

impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::WrongMagic(x) => {
                write!(fmt, "Wrong magic, found {} instead", x)
            }
            ParseError::IoError(x) => {
                write!(fmt, "Io error: {}", x)
            }
            ParseError::InvalidConstantTableEntry(x) => {
                write!(fmt, "Encountered invalid type id {} in constant table", x)
            }
            ParseError::Utf8Error(x) => {
                write!(fmt, "Encountered invalid utf8: {}", x)
            }
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::IoError(inner) => {
                Some(inner)
            }
            ParseError::Utf8Error(inner) => {
                Some(inner)
            }
            _ => {
                None
            }
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(err: FromUtf8Error) -> Self {
        ParseError::Utf8Error(err)
    }
}

pub trait ByteParseable {
    fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseError> where Self: Sized {
        return Self::parse(&mut Cursor::new(bytes));
    }

    fn parse(bytes: &mut impl Read) -> Result<Self, ParseError> where Self: Sized;

    fn parse_array(bytes: &mut impl Read, amount: usize) -> Result<Vec<Self>, ParseError> where Self: Sized {
        let mut res = Vec::with_capacity(amount);
        for _ in 0..amount {
            res.push(Self::parse(bytes)?);
        }
        return Ok(res);
    }
}

#[cfg(test)]
mod tests {
    use crate::class_file::{ByteParseable, ParseError};
    use std::io::{Read, Cursor};
    use crate::byte_util::BigEndianReadExt;

    #[derive(Eq, PartialEq, Debug)]
    struct Test(u8);

    impl ByteParseable for Test {
        fn parse(bytes: &mut impl Read) -> Result<Self, ParseError> where Self: Sized {
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