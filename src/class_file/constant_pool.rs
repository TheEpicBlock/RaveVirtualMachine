use std::io::Read;
use crate::byte_util::{BigEndianReadExt, read_to_vec, ByteParseable};
use crate::class_file::constant_pool::ConstantPoolInfo::Class;
use crate::class_file::parsing::ClassParseError;

/// See: https://docs.oracle.com/javase/specs/jvms/se16/html/jvms-4.html#jvms-4.4.1
#[derive(Debug)]
pub enum ConstantPoolInfo {
    Class(u16),
    FieldRef(u16, u16),
    MethodRef(u16, u16),
    InterfaceMethodRef(u16, u16),
    StringInfo(u16),
    IntegerInfo(u32),
    FloatInfo(f32),
    LongInfo(u64),
    DoubleInfo(f64),
    NameAndTypeInfo(u16, u16),
    Utf8Info(String),
    MethodHandleInfo(u8, u16),
    MethodTypeInfo(u16),
    DynamicInfo(u16, u16),
    InvokeDynamicInfo(u16, u16),
    ModuleInfo(u16),
    PackageInfo(u16),
}

impl ByteParseable<ClassParseError> for ConstantPoolInfo {
    fn parse(mut bytes: &mut impl Read) -> Result<Self, ClassParseError> {
        let tag = bytes.read_u8()?;
        match tag {
            7 => {
                Ok(ConstantPoolInfo::Class(bytes.read_u16()?))
            }
            9 => {
                Ok(ConstantPoolInfo::FieldRef(bytes.read_u16()?, bytes.read_u16()?))
            }
            10 => {
                Ok(ConstantPoolInfo::MethodRef(bytes.read_u16()?, bytes.read_u16()?))
            }
            11 => {
                Ok(ConstantPoolInfo::InterfaceMethodRef(bytes.read_u16()?, bytes.read_u16()?))
            }
            8 => {
                Ok(ConstantPoolInfo::Class(bytes.read_u16()?))
            }
            3 => {
                Ok(ConstantPoolInfo::IntegerInfo(bytes.read_u32()?))
            }
            4 => {
                Ok(ConstantPoolInfo::FloatInfo(bytes.read_f32()?))
            }
            5 => {
                Ok(ConstantPoolInfo::LongInfo(bytes.read_u64()?))
            }
            6 => {
                Ok(ConstantPoolInfo::DoubleInfo(bytes.read_f64()?))
            }
            12 => {
                Ok(ConstantPoolInfo::NameAndTypeInfo(bytes.read_u16()?, bytes.read_u16()?))
            }
            1 => {
                let len = bytes.read_u16()?;
                let vec = read_to_vec(bytes, len as usize)?;
                // TODO doesn't respect custom string format
                Ok(ConstantPoolInfo::Utf8Info(String::from_utf8(vec)?))
            }
            15 => {
                Ok(ConstantPoolInfo::MethodHandleInfo(bytes.read_u8()?, bytes.read_u16()?))
            }
            16 => {
                Ok(ConstantPoolInfo::MethodTypeInfo(bytes.read_u16()?))
            }
            17 => {
                Ok(ConstantPoolInfo::DynamicInfo(bytes.read_u16()?, bytes.read_u16()?))
            }
            18 => {
                Ok(ConstantPoolInfo::InvokeDynamicInfo(bytes.read_u16()?, bytes.read_u16()?))
            }
            19 => {
                Ok(ConstantPoolInfo::ModuleInfo(bytes.read_u16()?))
            }
            20 => {
                Ok(ConstantPoolInfo::PackageInfo(bytes.read_u16()?))
            }
            _ => {
                Err(ClassParseError::InvalidConstantTableEntry(tag))
            }
        }
    }
}

impl ConstantPoolInfo {
    fn get_tag(&self) -> u8 {
        match self {
            Class(_) => { 7 }
            ConstantPoolInfo::FieldRef(_, _) => { 9 }
            ConstantPoolInfo::MethodRef(_, _) => { 10 }
            ConstantPoolInfo::InterfaceMethodRef(_, _) => { 11 }
            ConstantPoolInfo::StringInfo(_) => { 8 }
            ConstantPoolInfo::IntegerInfo(_) => { 3 }
            ConstantPoolInfo::FloatInfo(_) => { 4 }
            ConstantPoolInfo::LongInfo(_) => { 5 }
            ConstantPoolInfo::DoubleInfo(_) => { 6 }
            ConstantPoolInfo::NameAndTypeInfo(_, _) => { 12 }
            ConstantPoolInfo::Utf8Info(_) => { 1 }
            ConstantPoolInfo::MethodHandleInfo(_, _) => { 15 }
            ConstantPoolInfo::MethodTypeInfo(_) => { 16 }
            ConstantPoolInfo::DynamicInfo(_, _) => { 17 }
            ConstantPoolInfo::InvokeDynamicInfo(_, _) => { 18 }
            ConstantPoolInfo::ModuleInfo(_) => { 19 }
            ConstantPoolInfo::PackageInfo(_) => { 20 }
        }
    }
}