use classfile_parser::class_file::ClassFile;
use crate::ClassLoader;

pub struct SimpleClassLoader {
    base: ClassFile
}

impl ClassLoader for SimpleClassLoader {
    fn load(&self, _: &str) -> ClassFile {
        self.base.clone()
    }
}

impl SimpleClassLoader {
    pub fn new(base: ClassFile) -> Self {
        SimpleClassLoader {
            base: base
        }
    }
}