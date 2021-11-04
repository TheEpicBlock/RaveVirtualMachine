use classfile_parser::class_file::ClassFile;
use crate::ClassLoader;

pub struct SimpleClassLoader {
    base: ClassFile
}

impl ClassLoader for SimpleClassLoader {

}

impl SimpleClassLoader {
    pub fn new(base: ClassFile) -> Self {
        SimpleClassLoader {
            base
        }
    }
}