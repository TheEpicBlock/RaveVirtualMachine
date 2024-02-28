use std::collections::HashMap;


use inkwell::types::{IntType, AnyType, AnyTypeEnum};
use vm_core::{JitCompiler, ClassShell};
use vm_core::class_store::{MethodData, DescriptorEntry};
use vm_core::classfile_util::ConstantPoolExtensions;
use classfile_parser::constant_pool::{ConstantPool, types, ConstantPoolEntry};
use classfile_parser::bytecode::Instruction;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::{OptimizationLevel, AddressSpace};

pub struct LlvmJitCompiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl Default for CraneliftJitCompiler {
    fn default() -> Self {
        Self {
        }
    }
}

impl JitCompiler for LlvmJitCompiler {
    type ClassId = ClassId;
    type MethodId = MethodId;
    type ClassShell = LlvmClass;

    fn load(&mut self, classfile: classfile_parser::class_file::ClassFile) -> Result<ClassId,()> {
        let constant_pool = &classfile.constant_pool;
        let this_class = constant_pool.get_as::<types::Class>(classfile.this_class).ok_or(())?;
        let fullname = constant_pool.get_as_string(this_class.name_index).ok_or(())?.to_string();
        
        self.classes.push(LlvmClass::try_from(classfile)?);
        let id = ClassId(self.classes.len()-1);
        self.names_to_ids.insert(fullname, id);

        return Ok(id);
    }

    fn get(&self, id: ClassId) -> Result<&Self::ClassShell,()> {
        Ok(&self.classes[id.0])
    }

    fn run(&mut self, class: ClassId, method: Self::MethodId) {
        let class = &self.classes[class.0];
        
        
    }
}

pub struct LlvmClass {
    constant_pool: Vec<ConstantPoolEntry>,
    package: String,
    name: String,
    methods: Vec<LlvmMethod>,
}

#[derive(Clone, Copy)]
pub struct ClassId(usize);

#[derive(Clone, Copy)]
pub struct MethodId(usize);

pub struct LlvmMethod {
    pub data: MethodData
}

impl<'a> ClassShell for LlvmClass {
    type Method = MethodId;

    fn find_main(&self) -> Option<Self::Method> {
        let method_index = self.methods.iter().enumerate().find(|m| m.1.data.is_main())?.0;
        Some(MethodId(method_index))
    }

    fn get_method(&self, name: &str, descriptor: &str) -> Option<Self::Method> {
        let method_index = self.methods.iter().enumerate().find(|m| m.1.data.name == name && m.1.data.descriptor == descriptor)?.0;
        Some(MethodId(method_index))
    }
}

trait IntoType {
    fn to_type(&self, ctx: &'ctx Context) -> AnyTypeEnum<'ctx>;
    // fn to_abi(&self, target: TargetFrontendConfig) -> AbiParam {
    //     AbiParam::new(self.to_type(target))
    // }
}

impl IntoType for DescriptorEntry {
    fn to_type(&self, ctx: &'ctx Context) -> ctypes::Type {
        // TODO ptr types shouldn't be to ints
        match self {
            DescriptorEntry::Class(_) => ctx.i8_type().ptr_type(AddressSpace::default()),
            DescriptorEntry::Byte => ctx.i8_type(),
            DescriptorEntry::Char => ctx.i8_type(),
            DescriptorEntry::Double => ctx.f64_type(),
            DescriptorEntry::Float => ctx.f32_type(),
            DescriptorEntry::Int => ctx.i32_type(),
            DescriptorEntry::Long => ctx.i64_type(),
            DescriptorEntry::Short => ctx.i16_type(),
            DescriptorEntry::Boolean => ctx.bool_type(),
            DescriptorEntry::Void => ctx.void_type(),
            DescriptorEntry::Array(_) => ctx.i8_type().ptr_type(AddressSpace::default()),
        }
    }
}

impl TryFrom<ClassFile> for LlvmClass {
    type Error = ();

    fn try_from(classfile: ClassFile) -> Result<Self, Self::Error> {
        let constant_pool = classfile.constant_pool;
        let this_class = constant_pool.get_as::<types::Class>(classfile.this_class).ok_or(())?;
        let fullname = constant_pool.get_as_string(this_class.name_index).ok_or(())?.to_string();
        let name = fullname.rsplit_once("/").unwrap_or(("", &fullname));
        
        let methods = classfile.methods.into_iter().map(|m| LlvmMethod::from_info(m, &constant_pool).unwrap()).collect(); // FIXME something better than unwrap pls

        Ok(LlvmClass {
            constant_pool: constant_pool,
            package: name.0.to_string(),
            name: name.1.to_string(),
            methods: methods
        })
    }
}

impl LlvmMethod {
    fn from_info(info: MethodInfo, constant_pool: &impl ConstantPool) -> Result<Self, ()> {
        Ok(LlvmMethod {  
            data: MethodData::from_info(info, constant_pool)?
        })
    }
}

#[cfg(test)]
mod tests {
}
