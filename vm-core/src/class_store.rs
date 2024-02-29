use std::str::Chars;
use classfile_parser::class_file::{MethodInfo, MethodAccessFlags};
use classfile_parser::constant_pool::ConstantPool;
use bitflags::bitflags;
use classfile_parser::attributes::CodeAttribute;
use crate::class_store::DescriptorEntry::{Array, Boolean, Byte, Char, Double, Float, Int, Long, Short, Void};
use crate::classfile_util::ConstantPoolExtensions;

bitflags! {
    pub struct MethodFlags: u8 {
        const STATIC = 0b00000001;
        const FINAL  = 0b00000010;
        const SYNCHRONISED = 0b00001000;
        // const BRIDGE = 0x0040;
        // const VARARGS = 0x0080;
        // const NATIVE = 0x0100;
        // const ABSTRACT = 0x0400;
        // const STRICT = 0x0800;
        // const SYNTHETIC = 0x1000;
    }
}

impl MethodFlags {
    pub fn from_access_flags(access_flags: &MethodAccessFlags) -> Self {
        let mut base = MethodFlags::empty();
        if access_flags.contains(MethodAccessFlags::STATIC) {
            base |= MethodFlags::STATIC;
        }
        if access_flags.contains(MethodAccessFlags::FINAL) {
            base |= MethodFlags::FINAL;
        }
        if access_flags.contains(MethodAccessFlags::SYNCHRONISED) {
            base |= MethodFlags::SYNCHRONISED;
        }
        return base;
    }
}

pub enum Visibility {
    Public,
    Protected,
    Private,
}

impl Visibility {
    pub fn from_flags(flags: &MethodAccessFlags) -> Self {
        if flags.contains(MethodAccessFlags::PUBLIC) {
            return Self::Public;
        } else if flags.contains(MethodAccessFlags::PROTECTED) {
            return Self::Protected;
        } else if flags.contains(MethodAccessFlags::PRIVATE) {
            return Self::Private;
        }
        Self::Private // Default to private
    }
}

pub struct MethodData<> {
    pub name: String,
    pub descriptor: String,
    pub visibility: Visibility,
    pub flags: MethodFlags,
    pub code: CodeAttribute,
}

#[derive(PartialEq, Debug)]
pub enum DescriptorEntry {
    Class(String),
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    Void,
    Array(Box<DescriptorEntry>)
}

impl DescriptorEntry {
    pub fn byte_size(&self) -> u64 {
        match self {
            DescriptorEntry::Class(_) => todo!(),
            Byte => 1,
            Char => 1,
            Double => 8,
            Float => 4,
            Int => 4,
            Long => 8,
            Short => 2,
            Boolean => 1,
            Void => 0,
            Array(_) => todo!(),
        }
    }
}

impl MethodData {
    pub fn from_info(method_info: MethodInfo, constant_pool: &impl ConstantPool) -> Result<Self, ()> {
        let name = constant_pool.get_as_string(method_info.name_index).ok_or(())?.to_string();
        let descriptor = constant_pool.get_as_string(method_info.descriptor).ok_or(())?.to_string();
        let visibility = Visibility::from_flags(&method_info.access_flags);
        let flags = MethodFlags::from_access_flags(&method_info.access_flags);
        let mut code = Option::None;

        for attribute in method_info.attributes {
            if let classfile_parser::attributes::AttributeEntry::Code(code_attribute) = attribute {
                code = Some(code_attribute);
            }
        }

        Ok(MethodData {
            name,
            descriptor,
            visibility,
            flags,
            code: code.unwrap()
        })
    }

    /// Checks if the function matches the definition of a main function
    pub fn is_main(&self) -> bool {
        self.name == "main" && self.descriptor == "([Ljava/lang/String;)V" && self.is_public() && self.is_static()
    }

    pub fn is_static(&self) -> bool {
        self.flags.contains(MethodFlags::STATIC)
    }

    pub fn is_public(&self) -> bool {
        matches!(self.visibility, Visibility::Public)
    }

    pub fn parse_descriptors(&self) -> (Vec<DescriptorEntry>, DescriptorEntry) {
        let chars =  &mut self.descriptor.chars();
        assert_eq!(chars.next(), Some('('));

        let mut acc = vec![];
        loop {
            let n = MethodData::parse_next_descriptor(chars);
            match n {
                None => {
                    return (acc, MethodData::parse_next_descriptor(chars).unwrap());
                }
                Some(x) => {
                    assert_ne!(x, Void, "Can't use void as a parameter");
                    acc.push(x);
                }
            }
        }
    }

    fn parse_next_descriptor(chars: &mut Chars) -> Option<DescriptorEntry> {
        match chars.next() {
            None => {
                None
            }
            Some(x) => {
                Some(match x {
                    'B' => Byte,
                    'C' => Char,
                    'D' => Double,
                    'F' => Float,
                    'I' => Int,
                    'J' => Long,
                    'L' => DescriptorEntry::Class(chars.take_while(|c| c != &';').collect()),
                    'S' => Short,
                    'Z' => Boolean,
                    'V' => Void,
                    '[' => Array(Box::new(MethodData::parse_next_descriptor(chars).unwrap())),
                    ')' => {
                        return None; // TODO this can be better
                    }
                    _ => {
                        //TODO proper failure
                        panic!("Invalid descriptor")
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::class_store::*;
    use crate::class_store::DescriptorEntry::Class;
    use crate::class_store::MethodData;

    #[test]
    fn method_descriptor() {
        let method = MethodData {
            name: "".to_string(),
            descriptor: "([Ljava/lang/String;DSZ)V".to_string(),
            visibility: Visibility::Public,
            flags: MethodFlags { bits: 0 },
            code: CodeAttribute {
                max_stack: 0,
                max_locals: 0,
                code: vec![],
                exception_table: vec![],
                attributes: vec![]
            }
        };

        assert_eq!(method.parse_descriptors(), (
            vec![
                Array(Box::new(Class("java/lang/String".to_string()))),
                Double,
                Short,
                Boolean
            ],
            Void
            ));
    }
}