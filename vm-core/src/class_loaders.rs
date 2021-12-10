use classfile_parser::class_file::ClassFile;
use crate::ClassLoader;

pub struct SimpleClassLoader {
    base: Option<ClassFile>
}

impl ClassLoader for SimpleClassLoader {
    fn load(&mut self, class: &str) -> ClassFile {
        let base = self.base.take();
        match base {
            None => {
                panic!("Called load twice");
            }
            Some(file) => {
                return file;
            }
        }
    }
}

impl SimpleClassLoader {
    pub fn new(base: ClassFile) -> Self {
        SimpleClassLoader {
            base: Some(base)
        }
    }
}