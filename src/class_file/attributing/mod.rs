pub mod attributes;
pub mod attribute_parsing;

use crate::class_file::constant_pool::{ConstantPoolEntry, ConstantPool};
use crate::class_file::parsing::{ParsedClass, MethodInfo, AttributeInfo};
use crate::class_file::{BasicClass, Stage};
use std::convert::{TryFrom, TryInto};
use thiserror::Error;
use crate::class_file::attributing::attribute_parsing::ParsedAttribute;
use std::fs::read_to_string;

#[derive(Error, Debug)]
pub enum AttributingError {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid Constant Pool index: {0}")]
    InvalidConstantPoolIndex(u16),
    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(u8),
}

bitflags! {
    pub struct ClassAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
        const MODULE = 0x8000;
    }
}

bitflags! {
    pub struct MethodAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONISED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABSTRACT = 0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
    }
}

pub trait TryAttributeFrom<T> {
    fn parse(info: T, pool: &impl ConstantPool) -> Result<Self, AttributingError> where Self: Sized;
}

pub struct AttributedClass {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantPoolEntry>,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    // pub interfaces: Vec<InterfaceInfo>,
    // pub fields: Vec<FieldInfo>,
    pub methods: Vec<AttributedMethod>,
    pub attributes: Vec<ParsedAttribute>
}

pub struct AttributedMethod {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<ParsedAttribute>,
}

impl BasicClass for AttributedClass {
    fn get_stage() -> Stage {
        Stage::Parsed
    }
}

impl TryFrom<ParsedClass> for AttributedClass {
    type Error = AttributingError;

    fn try_from(value: ParsedClass) -> Result<Self, Self::Error> {
        return Ok(AttributedClass {
            minor_version: value.minor_version,
            major_version: value.major_version,
            access_flags: ClassAccessFlags::from_bits_truncate(value.access_flags),
            this_class: value.this_class,
            super_class: value.super_class,
            methods: convert_vec(value.methods, &value.constant_pool)?,
            attributes: parse_attributes(value.attributes, &value.constant_pool)?,
            constant_pool: value.constant_pool,
        });
    }
}

fn convert_vec<T, B: TryAttributeFrom<T>>(input: Vec<T>, pool: &impl ConstantPool) -> Result<Vec<B>, AttributingError> {
    let mut new_vec = Vec::with_capacity(input.len());

    for object in input {
        new_vec.push(B::parse(object, pool)?);
    }

    return Ok(new_vec);
}

fn parse_attributes(input: Vec<AttributeInfo>, pool: &impl ConstantPool) -> Result<Vec<ParsedAttribute>, AttributingError> {
    let mut new_vec = Vec::with_capacity(input.len());

    for object in input {
        let optional_attribute = ParsedAttribute::from(object, pool)?;
        if let Some(attribute) = optional_attribute {
            new_vec.push(attribute);
        }
    }

    return Ok(new_vec);
}

impl TryAttributeFrom<MethodInfo> for AttributedMethod {
    fn parse(info: MethodInfo, pool: &impl ConstantPool) -> Result<Self, AttributingError> where Self: Sized {
        return Ok(AttributedMethod {
            access_flags: MethodAccessFlags::from_bits_truncate(info.access_flags),
            name_index: info.name_index,
            descriptor_index: info.descriptor_index,
            attributes: parse_attributes(info.attributes, pool)?
        })
    }
}