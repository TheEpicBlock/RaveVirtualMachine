use core::fmt;
use std::error::Error;
use std::io::{Cursor, Read};
use crate::byte_util::BigEndianReadExt;
use std::iter::Zip;
use std::string::FromUtf8Error;

mod parsing;
mod constantpool;

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

    fn parse(bytes: &mut impl Read) -> Result<Self, ParseError> where Self: Sized ;
}

impl<T: ByteParseable> ByteParseable for Vec<T>{
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ParseError> {
        let total = bytes.read_u16()? as usize;
        let mut res: Vec<T> = Vec::with_capacity(total);
        for _ in 0..total {
            res.push(T::parse(bytes)?);
        }
        return Ok(res);
    }
}

mod tests {
    use super::*;

    impl ByteParseable for u8 {
        fn parse(bytes: &mut impl Read) -> Result<Self, ParseError> where Self: Sized {
            Ok(bytes.read_u8()?)
        }
    }

    #[test]
    fn vector_byte_parse() {
        let vector: Vec<u8> = vec![1,2,3,5,8];
        let mut byte_vector = vec![0, vector.len() as u8];
        byte_vector.append(&mut vector.clone());
        println!("{}", byte_vector.len());

        //Vector is now out original list of numbers. And byte_vector is the same but with the length appended at the front as a u16.
        assert_eq!(vector, Vec::parse(&mut Cursor::new(byte_vector)).unwrap())
    }
}