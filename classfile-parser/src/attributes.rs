use crate::constant_pool::{types, ConstantPool};
use std::io::{Cursor, Read};
use crate::ClassParseError;
use crate::byte_util::{ByteParseable, BigEndianReadExt, read_to_vec};
use crate::gen_parseable;
use crate::bytecode::Instruction;

macro_rules! gen_attribute_parser {
    (
        pub enum $Name:ident {
            $(
                $Flag:ident($Type:ident) = $Value:expr,
            )+
        }
    ) => {
        pub enum $Name {
            $(
                $Flag($Type),
            )+
        }

        impl $Name {
            pub fn from(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Option<Self>, ClassParseError> {
                let name_index = bytes.read_u16()?;
                let name = pool.get_as::<types::Utf8Info>(name_index); // Look up the index in the constant pool
                match name {
                    Some(string) => {
                        match &string.inner[..] {
                            // For each known name, we generate a match statement
                            $(
                                $Value => {
                                    Ok(Some($Name::$Flag(Attribute::parse(bytes, pool)?)))
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
                }
            }
        }
    }
}

trait Attribute {
    fn parse_bytes(bytes: &[u8], pool: &impl ConstantPool) -> Result<Self, ClassParseError> where Self: Sized {
        return Self::parse(&mut Cursor::new(bytes), pool);
    }

    fn parse(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Self, ClassParseError> where Self: Sized;
}

impl<T: ByteParseable> Attribute for T {
    fn parse(bytes: &mut impl Read, _: &impl ConstantPool) -> Result<Self, ClassParseError> {
        Self::parse(bytes)
    }
}

gen_attribute_parser!(
    pub enum AttributeEntry {
        ConstantValue(ConstantValueAttribute) = "ConstantValue",
        Code(CodeAttribute) = "Code",
    }
);

pub fn parse_attribute_array(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Vec<AttributeEntry>, ClassParseError> {
    let amount = bytes.read_u16()?;

    let mut result = Vec::with_capacity(amount as usize);
    for _ in 0..amount {
        let optional_entry = AttributeEntry::from(bytes, pool)?;
        match optional_entry {
            Some(entry) => result.push(entry),
            None => {} // Ignore unrecognized attributes
        }
    }
    return Ok(result);
}

// Attributes

gen_parseable! {
    pub struct ConstantValueAttribute {
        value_index: u16,
    }
}

pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<Instruction>,
    pub exception_table: Vec<u8>,
    pub attributes: Vec<AttributeEntry>,
}

impl Attribute for CodeAttribute {
    fn parse(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Self, ClassParseError> where Self: Sized {
        let max_stack = bytes.read_u16()?;
        let max_locals = bytes.read_u16()?;

        let bytecode_size = bytes.read_u32()?;
        let mut bytecode_bytes = bytes.take(bytecode_size as u64);

        let mut bytecode = Vec::new();
        while bytecode_bytes.limit() != 0 {
            bytecode.push(ByteParseable::parse(&mut bytecode_bytes)?);
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
    use crate::class_file::constant_pool::{ConstantPoolEntry, Utf8Info};
    use crate::class_file::parsing::AttributeInfo;
    use crate::class_file::attributing::attribute_parsing::ParsedAttribute;
    use crate::class_file::attributing::AttributingError;
    use crate::attributes::AttributeEntry;

    #[test]
    fn parse_constant_value() {
        let pool = vec![
            ConstantPoolEntry::Utf8Info(Utf8Info { inner: "ConstantValue".to_owned() })
        ];

        let bytes = vec![
            1, //name index
            5, 6
        ];

        let parsed = ParsedAttribute::from(&bytes, &pool).unwrap().unwrap();
        assert!(matches!(parsed, ParsedAttribute::ConstantValue(_)))
    }

    #[test]
    fn parse_unknown() {
        let pool = vec![
            ConstantPoolEntry::Utf8Info(Utf8Info { inner: "Unknown Value".to_owned() })
        ];

        let attribute_info = AttributeInfo {
            name_index: 1,
            attribute: vec![5],
        };

        let parsed = ParsedAttribute::from(attribute_info, &pool).unwrap();
        assert!(matches!(parsed, None));
    }

    #[test]
    fn parse_invalid_index() {
        let pool = vec![
            ConstantPoolEntry::Utf8Info(Utf8Info { inner: "Unknown Value".to_owned() })
        ];

        let attribute_info = AttributeInfo {
            name_index: 233,
            attribute: vec![5],
        };

        let parsed = ParsedAttribute::from(attribute_info, &pool);
        if let Err(err) = parsed {
            assert!(matches!(err, AttributingError::InvalidConstantPoolIndex(233)));
        } else {
            panic!("Didn't receive error");
        }
    }
}