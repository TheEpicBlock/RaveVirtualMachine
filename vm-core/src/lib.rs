pub mod classfile_util;
mod class_store;
pub mod class_loaders;

use classfile_parser::class_file::ClassFile;
use crate::classfile_util::find_main;
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

    pub fn start(&mut self) {
        let main_method = find_main(&self.class).expect("couldn't find main method"); //FIXME shouldn't be a panic

    }
}

pub trait ClassLoader {

}

pub trait JitCompiler {

}

#[cfg(test)]
mod tests {
}
