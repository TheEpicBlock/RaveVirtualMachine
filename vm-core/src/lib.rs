pub mod classfile_util;
pub mod class_store;
pub mod class_loaders;
pub mod types;
/// Interop between rust functions and java ones
pub mod interop;

use std::{collections::HashMap, mem::transmute_copy};

use class_store::{ClassData, ClassStore, ClassStoreIsh, LoadedMethodRef};
use classfile_parser::class_file::ClassFile;
use interop::JavaCompatibleFunction;

pub struct VirtualMachine<L: ClassLoader, T: JitCompiler> {
    class_store: ClassStore<T>,
    class_loader: L,
    jit_engine: T,
}

impl<L: ClassLoader, T: JitCompiler> VirtualMachine<L, T> {
    pub fn new(class_loader: L, jit_engine: T) -> Self {
        VirtualMachine {
            class_store: Default::default(),
            class_loader,
            jit_engine,
        }
    }

    pub fn run(&mut self, class: &str, name: &str, descriptor: &str) -> Result<(),()> {
        let classfile = self.class_loader.load(class);
        let jit_data = self.jit_engine.load(&classfile)?;
        let classref = self.class_store.store(ClassData {
            java_class: classfile,
            jit_data
        });
        let method = self.class_store.retrieve_method_ref(classref, name, descriptor).ok_or(())?;

        self.jit_engine.get_fn_pointer(method, self.get_resolver());

        Ok(())
    }

    pub fn get_fn_pointer<F: JavaCompatibleFunction>(&mut self, class: &str, name: &str) -> Result<F, ()> {
        let fn_ptr = self.get_fn_pointer_raw(class, name, F::DESCRIPTOR)?;
        
        // Safe as long as the descriptor of the function matches its signature
        unsafe {
            Ok(transmute_copy(&fn_ptr))
        }
    }

    pub fn get_fn_pointer_raw(&mut self, class: &str, name: &str, descriptor: &str) -> Result<usize, ()> {
        // TODO encode descriptor in JavaCompatibleFunction
        let classfile = self.class_loader.load(class);
        let jit_data = self.jit_engine.load(&classfile)?;
        let classref = self.class_store.store(ClassData {
            java_class: classfile,
            jit_data
        });
        let method = self.class_store.retrieve_method_ref(classref, name, descriptor).ok_or(())?;

        return Ok(self.jit_engine.get_fn_pointer(method, self.get_resolver()));
    }

    fn get_resolver(&self) -> &impl ClassResolver<T> {
        &self.class_store
    }
}

pub trait ClassLoader {
    fn load(&self, class: &str) -> ClassFile;
}

pub trait JitCompiler: Sized {
    type ClassData;

    fn load(&mut self, class: &ClassFile) -> Result<Self::ClassData,()>;

    fn get_fn_pointer(&self, method: LoadedMethodRef, resolver: &impl ClassResolver<Self>) -> usize;
}

pub trait ClassResolver<J: JitCompiler>: ClassStoreIsh<J> {
}

impl<J: JitCompiler> ClassResolver<J> for ClassStore<J> {

}

pub trait ClassShell {
    type Method;

    fn find_main(&self) -> Option<Self::Method>;

    fn get_method(&self, name: &str, descriptor: &str) -> Option<Self::Method>;
}

#[cfg(test)]
mod tests {
}
