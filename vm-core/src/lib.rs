pub mod classfile_util;
pub mod class_store;
pub mod class_loaders;

use classfile_parser::class_file::ClassFile;
use crate::class_store::{MethodData};

pub struct VirtualMachine<L: ClassLoader, T: JitCompiler> {
    class_loader: L,
    jit_engine: T,
}

impl<L: ClassLoader, T: JitCompiler> VirtualMachine<L, T> {
    pub fn new(class_loader: L, jit_engine: T) -> Self {
        VirtualMachine {
            class_loader,
            jit_engine,
        }
    }

    pub fn start(&mut self, main: &str) -> Result<(),()> {
        let classfile = self.class_loader.load(main);
        self.jit_engine.load(classfile)?;

        let class = self.jit_engine.get(main)?;
        let main = class.find_main().ok_or(())?;

        self.jit_engine.run(main);

        Ok(())
    }
}

pub trait ClassLoader {
    fn load(&self, class: &str) -> ClassFile;
}

pub trait JitCompiler {
    type Method: MethodShell;
    type Class: ClassShell<Method = Self::Method>;

    fn load(&mut self, class: ClassFile) -> Result<(),()>;

    fn get(&self, name: &str) -> Result<&Self::Class, ()>;

    fn run(&mut self, method: &Self::Method);
}

pub trait ClassShell {
    type Method: MethodShell;

    fn find_main(&self) -> Option<&Self::Method>;
}

pub trait MethodShell {
    fn run(self);
}

#[cfg(test)]
mod tests {
}
