use std::io::{Cursor, Read};

use crate::class_file::{ParseError, ByteParseable};
use crate::byte_util::BigEndianReadExt;

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

pub struct ConstantPoolInfo {

}

pub struct InterfaceInfo {

}

pub struct FieldInfo {

}

pub struct MethodInfo {

}

pub struct AttributeInfo {

}

impl ByteParseable for ConstantPoolInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ParseError> {
        todo!()
    }
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

