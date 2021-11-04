use std::iter::Map;
use std::collections::HashMap;
use classfile_parser::class_file::ClassFile;
use classfile_parser::constant_pool::ConstantPool;
use classfile_parser::constant_pool::types;

pub(crate) struct ClassStore<'a> {
    classes: HashMap<String, Class<'a>>
}

impl Default for ClassStore {
    fn default() -> Self {
        todo!()
    }
}

impl<'a> ClassStore<'a> {
    pub fn add_from_classfile(&mut self, classfile: ClassFile) -> Result<(), ()> {
        let this_class = classfile.constant_pool.get_as::<types::Class>(classfile.this_class);
        
        Ok(())
    }
}

pub(crate) struct Class<'a> {
    package: &'a str,
    name: &'a str,
    functions: Vec<Function>,
}

pub(crate) struct Function {

}