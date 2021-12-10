pub mod classfile_util;
mod class_store;
pub mod class_loaders;

use classfile_parser::class_file::ClassFile;
use crate::class_store::ClassStore;

pub struct VirtualMachine<'a, L: ClassLoader, T: JitCompiler> {
    class_store: ClassStore<'a>,
    class_loader: L,
    jit_engine: T,
}

impl<'a, L: ClassLoader, T: JitCompiler> VirtualMachine<'a, L, T> {
    pub fn new(class_loader: L, jit_engine: T) -> Self {
        VirtualMachine {
            class_store: ClassStore::default(),
            class_loader,
            jit_engine,
        }
    }

    pub fn start(&mut self, main: &str) -> Result<(),()> {
        let classfile = self.class_loader.load(main);
        let class = self.class_store.add_from_classfile(classfile)?;
        class.find_main();

        Ok(())
    }
}

pub trait ClassLoader {
    fn load(&mut self, class: &str) -> ClassFile;
}

pub trait JitCompiler {

}

#[cfg(test)]
mod tests {
}
