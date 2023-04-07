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
        let class = self.jit_engine.load(classfile)?;

        let classShell = self.jit_engine.get(class)?;
        let main = classShell.find_main().ok_or(())?;

        self.jit_engine.run(class, main);

        Ok(())
    }

    pub fn run(&mut self, class: &str, name: &str, descriptor: &str) -> Result<(),()> {
        let classfile = self.class_loader.load(class);
        let class = self.jit_engine.load(classfile)?;

        let classShell = self.jit_engine.get(class)?;
        let main = classShell.get_method(name, descriptor).unwrap();

        self.jit_engine.run(class, main);

        Ok(())
    }
}

pub trait ClassLoader {
    fn load(&self, class: &str) -> ClassFile;
}

pub trait JitCompiler {
    type MethodId: Copy + Clone;
    type ClassId: Copy + Clone;
    type ClassShell: ClassShell<Method = Self::MethodId>;

    fn load(&mut self, class: ClassFile) -> Result<Self::ClassId,()>;

    fn get(&self, id: Self::ClassId) -> Result<&Self::ClassShell, ()>;

    fn run(&mut self, class: Self::ClassId, method: Self::MethodId);
}

pub trait ClassShell {
    type Method;

    fn find_main(&self) -> Option<Self::Method>;

    fn get_method(&self, name: &str, descriptor: &str) -> Option<Self::Method>;
}

#[cfg(test)]
mod tests {
}
