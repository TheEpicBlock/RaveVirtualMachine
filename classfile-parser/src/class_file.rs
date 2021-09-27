use crate::constant_pool::ConstantPoolEntry;
use crate::byte_util::{ByteParseable, BigEndianReadExt};
use std::io::Read;
use crate::ClassParseError;
use crate::attributes::AttributeEntry;

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

pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantPoolEntry>,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    // pub interfaces: Vec<InterfaceInfo>,
    // pub fields: Vec<FieldInfo>,
    pub methods: Vec<Method>,
    pub attributes: Vec<AttributeEntry>
}

pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeEntry>,
}

impl ByteParseable for ClassFile {
    fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> where Self: Sized {
        let magic = bytes.read_u32()?;
        if magic != 0xCAFEBABE {
            return Err(ClassParseError::WrongMagic(magic));
        }

        return ();
    }
}
