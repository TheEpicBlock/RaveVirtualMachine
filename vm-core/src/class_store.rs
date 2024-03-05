use std::str::Chars;
use classfile_parser::class_file::{ClassFile, MethodAccessFlags, MethodInfo};
use classfile_parser::constant_pool::types::{self, MethodRef};
use classfile_parser::constant_pool::ConstantPool;
use bitflags::bitflags;
use classfile_parser::attributes::CodeAttribute;
use crate::class_store::DescriptorEntry::{Array, Boolean, Byte, Char, Double, Float, Int, Long, Short, Void};
use crate::classfile_util::ConstantPoolExtensions;
use crate::JitCompiler;

pub struct ClassStore<J: JitCompiler> {
    class_store: Vec<ClassData<J>>,
}

#[derive(Clone, Copy)]
pub struct LoadedClassRef(usize);

#[derive(Clone, Copy)]
pub struct LoadedMethodRef{
    pub class_ref: LoadedClassRef,
    method_index: usize,
}

impl<J: JitCompiler> Default for ClassStore<J> {
    fn default() -> Self {
        Self { class_store: Default::default() }
    }
}

pub trait ClassStoreIsh<J: JitCompiler> {
    fn retrieve(&self, class: LoadedClassRef) -> &ClassData<J>;

    fn retrieve_method_ref(&self, class: LoadedClassRef, method_name: &str, method_desc: &str) -> Option<LoadedMethodRef> {
        let loaded_class = self.retrieve(class);
        loaded_class.java_class.methods.iter().enumerate().find(|(_i, method)| {
            let name = loaded_class.java_class.constant_pool.get_as_string(method.name_index).unwrap();
            let desc = loaded_class.java_class.constant_pool.get_as_string(method.descriptor).unwrap();
            return method_name == name && method_desc == desc;
        }).map(|(i, _method)| {
            LoadedMethodRef {
                class_ref: class,
                method_index: i
            }
        })
    }
}

impl<J: JitCompiler> ClassStoreIsh<J> for ClassStore<J> {
    fn retrieve(&self, class: LoadedClassRef) -> &ClassData<J> {
        &self.class_store[class.0]
    }
}

impl<J: JitCompiler> ClassStore<J> {
    pub fn store(&mut self, class: ClassData<J>) -> LoadedClassRef {
        let i = self.class_store.len();
        self.class_store.push(class);
        return LoadedClassRef(i);
    }
}

pub struct ClassData<J: JitCompiler> {
    pub java_class: ClassFile,
    pub jit_data: J::ClassData,
}

impl<J: JitCompiler> ClassData<J> {
    pub fn retrieve_method(&self, method: LoadedMethodRef) -> MethodData {
        // TODO assert that the ref is for the correct class
        return MethodData::from_info(&self.java_class.methods[method.method_index], &self.java_class.constant_pool).unwrap();
    }

    pub fn name(&self) -> &str {
        return self.java_class.constant_pool.get_as_string(self.java_class.constant_pool.get_as::<types::Class>(self.java_class.this_class).unwrap().name_index).unwrap();
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

pub struct MethodData<'class> {
    pub name: &'class str,
    pub descriptor: &'class str,
    pub visibility: Visibility,
    pub flags: MethodFlags,
    pub code: &'class CodeAttribute,
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

impl<'class> MethodData<'class> {
    pub fn from_info(method_info: &'class MethodInfo, constant_pool: &'class impl ConstantPool) -> Result<Self, ()> {
        let name = constant_pool.get_as_string(method_info.name_index).ok_or(())?;
        let descriptor = constant_pool.get_as_string(method_info.descriptor).ok_or(())?;
        let visibility = Visibility::from_flags(&method_info.access_flags);
        let flags = MethodFlags::from_access_flags(&method_info.access_flags);
        let mut code = Option::None;

        for attribute in &method_info.attributes {
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
    use classfile_parser::bytecode::Code;

    use crate::class_store::*;
    use crate::class_store::DescriptorEntry::Class;
    use crate::class_store::MethodData;

    #[test]
    fn method_descriptor() {
        let method = MethodData {
            name: "",
            descriptor: "([Ljava/lang/String;DSZ)V",
            visibility: Visibility::Public,
            flags: MethodFlags { bits: 0 },
            code: &CodeAttribute {
                max_stack: 0,
                max_locals: 0,
                code: Code::from_vec(vec![]),
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