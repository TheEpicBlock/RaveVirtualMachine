use std::io::{Cursor, Read};

use crate::class_file::{ParseError, ByteParseable};
use crate::byte_util::BigEndianReadExt;
use crate::class_file::constantpool::ConstantPoolInfo;

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
    pub attributes: Vec<AttributeInfo>
}

#[derive(Debug)]
pub struct InterfaceInfo {

}

#[derive(Debug)]
pub struct FieldInfo {

}

#[derive(Debug)]
pub struct MethodInfo {

}

#[derive(Debug)]
pub struct AttributeInfo {

}

impl ByteParseable for InterfaceInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ParseError> {
        todo!()
    }
}

impl ByteParseable for FieldInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ParseError> {
        todo!()
    }
}

impl ByteParseable for MethodInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ParseError> {
        todo!()
    }
}

impl ByteParseable for AttributeInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ParseError> {
        todo!()
    }
}

impl ByteParseable for ParsedClass {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ParseError> {
        let magic = bytes.read_u32()?;
        if magic != 0xCAFEBABE {
            return Err(ParseError::WrongMagic(magic));
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
            attributes: parse_default_array(bytes)?
        })
    }
}

/// Parses an array of parseables where the first u16 is the size
fn parse_default_array<T: ByteParseable>(bytes: &mut impl Read) -> Result<Vec<T>, ParseError> {
    let size = bytes.read_u16()? as usize;
    T::parse_array(bytes, size)
}

fn parse_constant_pool<T: ByteParseable>(bytes: &mut impl Read) -> Result<Vec<T>, ParseError> {
    let size = bytes.read_u16()? as usize - 1;
    T::parse_array(bytes, size)
}

#[cfg(test)]
mod tests {
    use crate::class_file::parsing::{ParsedClass, parse_default_array};
    use crate::class_file::{ByteParseable, ParseError};
    use std::io::{Cursor, Read, Seek};
    use crate::byte_util::BigEndianReadExt;

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
                panic!("Expected an error but result was ok: {:?}",x)
            }
            Err(inner) => {
                match inner {
                    ParseError::WrongMagic(0x00000000) => {
                        // Correct result
                    }
                    ParseError::WrongMagic(x) => {
                        panic!("Expected 0x00000000 but found: {}", x)
                    }
                    x => {
                        panic!("Expected a wrong magic error but found: {}", x)
                    }
                }
            }
        }
    }

    #[derive(Eq, PartialEq, Debug)]
    struct Test(u8);

    impl ByteParseable for Test {
        fn parse(bytes: &mut impl Read) -> Result<Self, ParseError> where Self: Sized {
            Ok(Test(bytes.read_u8()?))
        }
    }

    #[test]
    fn test_array_parse() {
        let bytes = vec![1,2,3,5,8];
        let mut tests = Vec::with_capacity(bytes.len());
        for i in &bytes { tests.push(Test(*i)); }

        let mut byte_vector = vec![0, bytes.len() as u8];
        byte_vector.append(&mut bytes.clone());
        println!("{}", byte_vector.len());

        //tests is now an original list of numbers. And byte_vector is the same but with the length appended at the front as a u16.
        assert_eq!(tests, parse_default_array(&mut Cursor::new(byte_vector)).unwrap())
    }
}