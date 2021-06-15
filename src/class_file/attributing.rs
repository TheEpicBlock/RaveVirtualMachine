use crate::class_file::constant_pool::ConstantPoolInfo;
use crate::class_file::{Stage, BasicClass};
use crate::class_file::parsing::ParsedClass;
use std::convert::{TryFrom, TryInto};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AttributingError {

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

struct AttributedClass {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantPoolInfo>,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    // pub interfaces: Vec<InterfaceInfo>,
    // pub fields: Vec<FieldInfo>,
    // pub methods: Vec<MethodInfo>,
    // pub attributes: Vec<AttributeInfo>
}

struct AnnotatedMethod {

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
            constant_pool: value.constant_pool,
            access_flags: ClassAccessFlags::from_bits_truncate(value.access_flags),
            this_class: value.this_class,
            super_class: value.super_class
        })
    }
}
