use std::io::{Cursor, Error, Read};

use crate::byte_util::ByteParseable;
use crate::byte_util::{read_to_vec, BigEndianReadExt};
use crate::class_file::constant_pool::ConstantPoolInfo;
use crate::class_file::{BasicClass, Stage};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClassParseError {
    #[error("Wrong magic value found in class file: {0}")]
    WrongMagic(u32),
    #[error("Invalid contant table entry: {0}")]
    InvalidConstantTableEntry(u8),
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Utf8 parsing error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
}

#[derive(Debug)]
pub struct ParsedClass {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantPoolInfo>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<InterfaceInfo>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

impl BasicClass for ParsedClass {
    fn get_stage() -> Stage {
        Stage::Parsed
    }
}

#[derive(Debug)]
pub struct InterfaceInfo {}

#[derive(Debug)]
pub struct FieldInfo {}

#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct AttributeInfo {
    pub name_index: u16,
    pub attribute: Vec<u8>,
}

impl ByteParseable<ClassParseError> for InterfaceInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ClassParseError> {
        todo!()
    }
}

impl ByteParseable<ClassParseError> for FieldInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ClassParseError> {
        todo!()
    }
}

impl ByteParseable<ClassParseError> for MethodInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ClassParseError> {
        return Ok(MethodInfo {
            access_flags: bytes.read_u16()?,
            name_index: bytes.read_u16()?,
            descriptor_index: bytes.read_u16()?,
            attributes: parse_default_array(bytes)?,
        });
    }
}

impl ByteParseable<ClassParseError> for AttributeInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ClassParseError> {
        let name_index = bytes.read_u16()?;
        let attribute_length = bytes.read_u32()? as usize;

        return Ok(AttributeInfo {
            name_index,
            attribute: read_to_vec(bytes, attribute_length)?,
        });
    }
}

impl ByteParseable<ClassParseError> for ParsedClass {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ClassParseError> {
        let magic = bytes.read_u32()?;
        if magic != 0xCAFEBABE {
            return Err(ClassParseError::WrongMagic(magic));
        }

        return Ok(ParsedClass {
            minor_version: bytes.read_u16()?,
            major_version: bytes.read_u16()?,
            constant_pool: parse_constant_pool(bytes)?, // The number here is one larger than you'd expect
            access_flags: bytes.read_u16()?,
            this_class: bytes.read_u16()?,
            super_class: bytes.read_u16()?,
            interfaces: parse_default_array(bytes)?,
            fields: parse_default_array(bytes)?,
            methods: parse_default_array(bytes)?,
            attributes: parse_default_array(bytes)?,
        });
    }
}

/// Parses an array of parseables where the first u16 is the size
fn parse_default_array<ERR: std::error::Error + From<Error>, T: ByteParseable<ERR>>(bytes: &mut impl Read) -> Result<Vec<T>, ERR> {
    let size = bytes.read_u16()? as usize;
    T::parse_array(bytes, size)
}

fn parse_array_u32<ERR: std::error::Error + From<Error>, T: ByteParseable<ERR>>(bytes: &mut impl Read, ) -> Result<Vec<T>, ERR> {
    let size = bytes.read_u32()? as usize;
    T::parse_array(bytes, size)
}

fn parse_constant_pool<ERR: std::error::Error + From<Error>, T: ByteParseable<ERR>>(bytes: &mut impl Read, ) -> Result<Vec<T>, ERR> {
    let size = bytes.read_u16()? as usize - 1;
    T::parse_array(bytes, size)
}

#[cfg(test)]
mod tests {
    use crate::byte_util::{BigEndianReadExt, ByteParseable};
    use crate::class_file::parsing::{parse_default_array, ClassParseError, ParsedClass};
    use std::io::{Cursor, Read, Seek};

    #[test]
    #[should_panic]
    fn invalid_file() {
        let bytes = &[0x00, 0x13, 0x67];
        ParsedClass::parse_bytes(bytes).unwrap();
    }

    #[test]
    fn test_magic() {
        let bytes = &[0x00, 0x00, 0x00, 0x00]; // 0x0000 != 0xCAFEBABE
        let result = ParsedClass::parse_bytes(bytes);
        match result {
            Ok(x) => {
                panic!("Expected an error but result was ok: {:?}", x)
            }
            Err(inner) => {
                match inner {
                    ClassParseError::WrongMagic(0x00000000) => {
                        // Correct result
                    }
                    ClassParseError::WrongMagic(x) => {
                        panic!("Expected 0x00000000 but found: {}", x)
                    }
                    x => {
                        panic!("Expected a wrong magic error but found: {:?}", x)
                    }
                }
            }
        }
    }

    #[derive(Eq, PartialEq, Debug)]
    struct Test(u8);

    impl ByteParseable<std::io::Error> for Test {
        fn parse(bytes: &mut impl Read) -> Result<Self, std::io::Error>
        where
            Self: Sized,
        {
            Ok(Test(bytes.read_u8()?))
        }
    }

    #[test]
    fn test_array_parse() {
        let bytes = vec![1, 2, 3, 5, 8];
        let mut tests = Vec::with_capacity(bytes.len());
        for i in &bytes {
            tests.push(Test(*i));
        }

        let mut byte_vector = vec![0, bytes.len() as u8];
        byte_vector.append(&mut bytes.clone());
        println!("{}", byte_vector.len());

        //tests is now an original list of numbers. And byte_vector is the same but with the length appended at the front as a u16.
        assert_eq!(
            tests,
            parse_default_array(&mut Cursor::new(byte_vector)).unwrap()
        )
    }
}
