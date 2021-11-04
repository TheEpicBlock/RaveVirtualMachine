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

pub fn find_main(class: &ClassFile) -> Option<&MethodInfo> {
    for method in &class.methods {
        let name = class.constant_pool.get_as_string(method.name_index);
        let descriptor = class.constant_pool.get_as_string(method.descriptor);

        if let Some(name) = name {
            if name == "main" {
                if let Some(descriptor) = descriptor {
                    if descriptor == "([Ljava/lang/String;)V" {
                        return Some(method);
                    }
                }
            }
        }
    }

    None
}