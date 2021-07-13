use crate::class_file::attributing::attributes::*;
use crate::class_file::parsing::AttributeInfo;
use crate::class_file::constant_pool::{ConstantPool, Utf8Info};
use crate::class_file::constant_pool::types;
use crate::class_file::attributing::AttributingError;
use crate::class_file::attributing::AttributingError::InvalidConstantPoolIndex;
use crate::class_file::attributing::TryAttributeFrom;

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

        impl ParsedAttribute {
            pub fn from(info: AttributeInfo, pool: &impl ConstantPool) -> Result<Option<Self>, AttributingError> {
                let name = pool.get_as::<types::Utf8Info>(info.name_index);
                match name {
                    Some(string) => {
                        match &string.inner[..] {
                            $(
                                $Value => {
                                    Ok(Some($Name::$Flag($Type::parse(info, pool)?)))
                                },
                            )+
                            _ => {
                                Ok(None)
                            }
                        }
                    },
                    None => {
                        Err(InvalidConstantPoolIndex(info.name_index))
                    }
                }
            }
        }
    }
}

gen_attribute_parser!(
    pub enum ParsedAttribute {
        ConstantValue(ConstantValueAttribute) = "ConstantValue",
    }
);

#[cfg(test)]
mod tests {
    use crate::class_file::constant_pool::{ConstantPoolEntry, Utf8Info};
    use crate::class_file::parsing::AttributeInfo;
    use crate::class_file::attributing::attribute_parsing::ParsedAttribute;
    use crate::class_file::attributing::AttributingError;

    #[test]
    fn parse_constant_value() {
        let pool = vec![
            ConstantPoolEntry::Utf8Info(Utf8Info { inner: "ConstantValue".to_owned() })
        ];

        let attribute_info = AttributeInfo {
            name_index: 0,
            attribute: vec![5, 6],
        };

        let parsed = ParsedAttribute::from(attribute_info, &pool).unwrap().unwrap();
        assert!(matches!(parsed, ParsedAttribute::ConstantValue(_)))
    }

    #[test]
    fn parse_unknown() {
        let pool = vec![
            ConstantPoolEntry::Utf8Info(Utf8Info { inner: "Unknown Value".to_owned() })
        ];

        let attribute_info = AttributeInfo {
            name_index: 0,
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