use std::borrow::Borrow;
use std::iter::Map;
use std::collections::HashMap;
use std::str::Chars;
use classfile_parser::class_file::{ClassFile, MethodInfo, MethodAccessFlags};
use classfile_parser::constant_pool::ConstantPool;
use classfile_parser::constant_pool::types;
use bitflags::bitflags;
use classfile_parser::attributes::CodeAttribute;
use crate::class_store::DescriptorEntry::{Array, Boolean, Byte, Char, Double, Float, Int, Long, Short, Void};
use crate::classfile_util::ConstantPoolExtensions;

pub struct ClassStore<METHODJITDATA: Default> {
    classes: HashMap<String, Class<METHODJITDATA>>,
}

impl<METHODJITDATA: Default> Default for ClassStore<METHODJITDATA> {
    fn default() -> Self {
        ClassStore {
            classes: HashMap::new(),
        }
    }
}

impl<METHODJITDATA: Default> ClassStore<METHODJITDATA> {
    pub fn add_from_classfile(&mut self, classfile: ClassFile) -> Result<&Class<METHODJITDATA>, ()> {
        let constant_pool = classfile.constant_pool;
        let this_class = constant_pool.get_as::<types::Class>(classfile.this_class).ok_or(())?;
        let fullname = constant_pool.get_as_string(this_class.name_index).ok_or(())?.to_string();

        let name = fullname.rsplit_once("/").ok_or(())?;
        let class = Class {
            package: name.0.to_string(),
            name: name.1.to_string(),
            methods: classfile.methods.into_iter().map(|m| Method::from_info(m, &constant_pool).unwrap()).collect() // FIXME something better than unwrap pls
        };

        let copy_of_fullname = fullname.clone();
        self.classes.insert(fullname, class);
        return Ok(&self.classes[&copy_of_fullname]);
    }
}

pub struct Class<METHODJITDATA: Default> {
    package: String,
    name: String,
    methods: Vec<Method<METHODJITDATA>>,
}

impl<METHODJITDATA: Default> Class<METHODJITDATA> {
    /// Locates a main method in this class if available
    pub fn find_main(&self) -> Option<&Method<METHODJITDATA>> {
        self.methods.iter().find(|method| method.is_main())
    }
}


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

pub struct Method<JITDATA: Default> {
    pub name: String,
    pub descriptor: String,
    pub visibility: Visibility,
    pub flags: MethodFlags,
    pub code: CodeAttribute,
    pub jit_data: JITDATA,
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

impl<JITDATA: Default> Method<JITDATA> {
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

        Ok(Method {
            name,
            descriptor,
            visibility,
            flags,
            code: code.unwrap(),
            jit_data: JITDATA::default(),
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
        let mut chars =  &mut self.descriptor.chars();
        assert_eq!(chars.next(), Some('('));

        let mut acc = vec![];
        loop {
            let n = Method::<JITDATA>::parse_next_descriptor(&mut chars);
            match n {
                None => {
                    return (acc, Method::<JITDATA>::parse_next_descriptor(&mut chars).unwrap());
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
                return None;
            }
            Some(x) => {
                return Some(match x {
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
                    '[' => Array(Box::new(Method::<JITDATA>::parse_next_descriptor(chars).unwrap())),
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
    use crate::Method;

    #[test]
    fn method_descriptor() {
        let method = Method {
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
            },
            jit_data: ()
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