use crate::constant_pool::{types, ConstantPool};
use std::io::Read;
use crate::ClassParseError;
use crate::byte_util::{ByteParseable, BigEndianReadExt, read_to_vec};
use crate::gen_parseable;
use crate::bytecode::Instruction;

macro_rules! gen_attribute_parser {
    (
        $(#[$Meta:meta])*
        pub enum $Name:ident {
            $(
                $Flag:ident($Type:ident) = $Value:expr,
            )+
        }
    ) => {
        $(#[$Meta])*
        pub enum $Name {
            $(
                $Flag($Type),
            )+
        }

        impl $Name {
            pub fn parse(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Option<Self>, ClassParseError> {
                let name_index = bytes.read_u16()?;
                let attribute_size = bytes.read_u64()?;

                let name = pool.get_as::<types::Utf8Info>(name_index); // Look up the index in the constant pool
                let result = match name {
                    Some(string) => {
                        match &string.inner[..] {
                            // For each known name, we generate a match statement
                            $(
                                $Value => {
                                    Ok(Some($Name::$Flag(Attribute::parse(bytes, attribute_size, pool)
                                        .map_err(|e| ClassParseError::AttributingError(string.inner.clone(), Box::new(e)))?)))
                                },
                            )+
                            _ => {
                                Ok(None)
                            }
                        }
                    },
                    None => {
                        Err(ClassParseError::InvalidConstantPoolIndex(name_index))
                    }
                };

                return result;
            }
        }
    }
}

gen_attribute_parser!(
    #[derive(Debug)]
    pub enum AttributeEntry {
        ConstantValue(ConstantValueAttribute) = "ConstantValue",
        Code(CodeAttribute) = "Code",
    }
);

pub fn parse_attribute_array(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Vec<AttributeEntry>, ClassParseError> {
    let amount = bytes.read_u16()?;

    let mut result = Vec::with_capacity(amount as usize);
    for _ in 0..amount {
        let optional_entry = AttributeEntry::parse(bytes, pool)?;
        match optional_entry {
            Some(entry) => result.push(entry),
            None => {} // Ignore unrecognized attributes
        }
    }
    return Ok(result);
}

// Attributes

gen_parseable! {
    #[derive(Debug)]
    pub struct ConstantValueAttribute {
        value_index: u16,
    }
}

#[derive(Debug)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<Instruction>,
    pub exception_table: Vec<u8>,
    pub attributes: Vec<AttributeEntry>,
}

trait Attribute {
    fn parse(bytes: &mut impl Read, expected_size: u64, pool: &impl ConstantPool) -> Result<Self, ClassParseError> where Self: Sized;
}

impl<T: ByteParseable> Attribute for T {
    fn parse(bytes: &mut impl Read, _expected_size: u64, _pool: &impl ConstantPool) -> Result<Self, ClassParseError> {
        Self::parse(bytes)
    }
}

impl Attribute for CodeAttribute {
    fn parse(bytes: &mut impl Read, _expected_size: u64, pool: &impl ConstantPool) -> Result<Self, ClassParseError> where Self: Sized {
        let max_stack = bytes.read_u16()?;
        let max_locals = bytes.read_u16()?;

        let bytecode_size = bytes.read_u32()?;
        let mut bytecode_bytes = bytes.take(bytecode_size as u64);

        let mut bytecode = Vec::new();
        while bytecode_bytes.limit() != 0 {
            bytecode.push(ByteParseable::parse(&mut bytecode_bytes)
                .map_err(|e| e.with_misc_context("parsing bytecode array"))?);
        }

        let exception_table_size = bytes.read_u16()?;
        let exception_table = read_to_vec(bytes, (exception_table_size * 8) as usize)?;

        let attributes = parse_attribute_array(bytes, pool)?;

        return Ok(CodeAttribute {
            max_stack,
            max_locals,
            code: bytecode,
            exception_table,
            attributes
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::constant_pool::{ConstantPoolEntry, Utf8Info};
    use crate::attributes::AttributeEntry;
    use std::io::Cursor;
    use crate::ClassParseError;

    #[test]
    fn parse_constant_value() {
        let pool = vec![
            ConstantPoolEntry::Utf8Info(Utf8Info { inner: "ConstantValue".to_owned() })
        ];

        let bytes = vec![
            0, 1, //name index
            0, 0, 0, 0, 0, 0, 0, 2, // length
            5, 6u8 // content
        ];

        let parsed = AttributeEntry::parse(&mut Cursor::new(bytes), &pool).unwrap().unwrap();
        assert!(matches!(parsed, AttributeEntry::ConstantValue(_)))
    }

    #[test]
    fn parse_unknown() {
        let pool = vec![
            ConstantPoolEntry::Utf8Info(Utf8Info { inner: "Unknown Value".to_owned() })
        ];

        let bytes = vec![
            0, 1, //name index
            0, 0, 0, 0, 0, 0, 0, 1, // length
            5u8 // content
        ];

        let parsed = AttributeEntry::parse(&mut Cursor::new(bytes), &pool).unwrap();
        assert!(matches!(parsed, None));
    }

    #[test]
    fn parse_invalid_index() {
        let pool = vec![
            ConstantPoolEntry::Utf8Info(Utf8Info { inner: "Unknown Value".to_owned() })
        ];

        let bytes = vec![
            0, 233, //name index
            0, 0, 0, 0, 0, 0, 0, 1, // length
            5u8
        ];

        let parsed = AttributeEntry::parse(&mut Cursor::new(bytes), &pool);
        if let Err(err) = parsed {
            assert!(matches!(err, ClassParseError::InvalidConstantPoolIndex(233)));
        } else {
            panic!("Didn't receive error");
        }
    }
}