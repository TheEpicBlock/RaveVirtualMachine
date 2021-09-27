use crate::constant_pool::{ConstantPoolEntry, ParseableWithCP, ConstantPool, parse_multiple_with_cp};
use crate::byte_util::{ByteParseable, BigEndianReadExt, parse_multiple};
use std::io::Read;
use crate::ClassParseError;
use crate::attributes::{AttributeEntry, parse_attribute_array};

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
    pub struct FieldAccessFlags: u16 {
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

#[derive(Debug)]
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantPoolEntry>,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeEntry>
}

#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: FieldAccessFlags,
    pub name_index: u16,
    /// Index in the constant pool
    pub descriptor: u16,
    pub attributes: Vec<AttributeEntry>
}

#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    /// Index in the constant pool
    pub descriptor: u16,
    pub attributes: Vec<AttributeEntry>,
}

impl ByteParseable for ClassFile {
    fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> where Self: Sized {
        let magic = bytes.read_u32()?;
        if magic != 0xCAFEBABE {
            return Err(ClassParseError::WrongMagic(magic));
        }

        let minor_version = ByteParseable::parse(bytes)?;
        let major_version = ByteParseable::parse(bytes)?;

        let constant_pool_size = bytes.read_u16()?;
        let constant_pool = parse_multiple(bytes, (constant_pool_size - 1) as usize)?;

        let access_flags = ClassAccessFlags::from_bits_truncate(bytes.read_u16()?);

        let this_class = ByteParseable::parse(bytes)?;
        let super_class = ByteParseable::parse(bytes)?;

        let interfaces_size = bytes.read_u16()?;
        let interfaces = parse_multiple(bytes, interfaces_size as usize)?;

        let fields_size = bytes.read_u16()?;
        let fields = parse_multiple_with_cp(bytes, &constant_pool, fields_size as usize)?;

        let methods_size = bytes.read_u16()?;
        let methods = parse_multiple_with_cp(bytes, &constant_pool, methods_size as usize)?;

        let attributes = parse_attribute_array(bytes, &constant_pool)?;

        return Ok(ClassFile {
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes
        });
    }
}

impl ParseableWithCP for FieldInfo {
    fn parse(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Self, ClassParseError> {
        Ok(FieldInfo {
            access_flags: FieldAccessFlags::from_bits_truncate(bytes.read_u16()?),
            name_index: ByteParseable::parse(bytes)?,
            descriptor: ByteParseable::parse(bytes)?,
            attributes: parse_attribute_array(bytes, pool)?
        })
    }
}

impl ParseableWithCP for MethodInfo {
    fn parse(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Self, ClassParseError> {
        Ok(MethodInfo {
            access_flags: MethodAccessFlags::from_bits_truncate(bytes.read_u16()?),
            name_index: ByteParseable::parse(bytes)?,
            descriptor: ByteParseable::parse(bytes)?,
            attributes: parse_attribute_array(bytes, pool)?
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_util::{BigEndianReadExt, ByteParseable};
    use std::io::{Cursor, Read, Seek};
    use crate::ClassParseError;
    use crate::class_file::ClassFile;

    #[test]
    #[should_panic]
    fn invalid_file() {
        let bytes = &[0x00, 0x13, 0x67];
        ClassFile::parse_bytes(bytes).unwrap();
    }

    #[test]
    fn test_magic() {
        let bytes = &[0x00, 0x00, 0x00, 0x00]; // 0x0000 != 0xCAFEBABE
        let result = ClassFile::parse_bytes(bytes);
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
}
