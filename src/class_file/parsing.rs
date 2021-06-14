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
            constant_pool: Vec::parse(bytes)?,
            access_flags: bytes.read_u16()?,
            this_class: bytes.read_u16()?,
            super_class: bytes.read_u16()?,
            interfaces: Vec::parse(bytes)?,
            fields: Vec::parse(bytes)?,
            methods: Vec::parse(bytes)?,
            attributes: Vec::parse(bytes)?
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::class_file::parsing::ParsedClass;
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
}