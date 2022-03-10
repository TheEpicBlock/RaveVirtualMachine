pub mod classfile_util;
pub mod class_store;
pub mod class_loaders;

use std::marker::PhantomData;
use classfile_parser::class_file::ClassFile;
use crate::class_store::{ClassStore, Method};

pub struct VirtualMachine<L: ClassLoader, T: JitCompiler> {
    class_store: ClassStore,
    class_loader: L,
    jit_engine: T,
}

impl<L: ClassLoader, T: JitCompiler> VirtualMachine<L, T> {
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
        let main = class.find_main().ok_or(())?;

        println!("{}", &main.name);
        for inst in &main.code.code {
            println!(" - {:?}", inst);
        }
        assert_eq!(main.descriptor, "([Ljava/lang/String;)V");

        Ok(())
    }
}

pub trait ClassLoader {
    fn load(&mut self, class: &str) -> ClassFile;
}

pub trait JitCompiler {
    fn compile(&mut self, method: &Method, class_store: &ClassStore);
}

#[cfg(test)]
mod tests {
}
