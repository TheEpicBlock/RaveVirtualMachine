use classfile_parser::class_file::{ClassFile, MethodInfo};
use classfile_parser::constant_pool::ConstantPool;
use classfile_parser::constant_pool::types;
use classfile_parser::attributes::{AttributeEntry, CodeAttribute};

pub trait ConstantPoolExtensions: ConstantPool {
    fn get_as_string(&self, index: u16) -> Option<&str> {
        self.get_as::<types::Utf8Info>(index).map(|v| v.inner.as_str())
    }
}

impl<R: ConstantPool + ?Sized> ConstantPoolExtensions for R {}

pub fn get_code_attribute(method: &MethodInfo) -> Option<&CodeAttribute> {
    for attribute in &method.attributes {
        if let AttributeEntry::Code(inner) = attribute {
            return Some(inner);
        }
    }
    None
}