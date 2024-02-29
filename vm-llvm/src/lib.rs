mod type_translation;

use std::collections::HashMap;
use std::mem;

use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, VoidType};
use inkwell::values::{AnyValue, BasicValueEnum, IntValue, PointerValue};
use type_translation::LlvmReturnType;
use vm_core::{JitCompiler, ClassShell};
use vm_core::class_store::{DescriptorEntry, MethodData};
use vm_core::classfile_util::ConstantPoolExtensions;
use classfile_parser::constant_pool::{ConstantPool, types, ConstantPoolEntry};
use classfile_parser::bytecode::Instruction;
use inkwell::builder::{self, Builder};
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use classfile_parser::class_file::{ClassFile, MethodInfo};

use crate::type_translation::IntoType;

pub struct LlvmJitCompiler {
    context: &'static Context,
    module: Module<'static>,
    builder: Builder<'static>,
    execution_engine: ExecutionEngine<'static>,

    names_to_ids: HashMap<String, ClassId>,
    classes: Vec<LlvmClass>
}

impl Default for LlvmJitCompiler {
    fn default() -> Self {
        // This is fine
        let ctx = Box::leak(Box::new(Context::create()));
        let m = ctx.create_module("main");
        let e = m.create_jit_execution_engine(OptimizationLevel::None).unwrap();
        Self {
            context: ctx,
            module: m,
            builder: ctx.create_builder(),
            execution_engine: e,

            names_to_ids: HashMap::new(),
            classes: Vec::new(),
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
        let method = &class.methods[method.0];
        let desc = method.data.parse_descriptors();
        
        let usize_type = self.context.ptr_sized_int_type(self.execution_engine.get_target_data(), None);
        let return_types: Vec<_> = desc.0.iter().map(|t| t.to_type(self.context).unwrap().to_meta()).collect();
        let ty = desc.1.to_type(self.context).unwrap().fn_type(&return_types, false);
        let function = self.module.add_function(&format!("{}-{}", class.name, method.data.name), ty, None);

        let block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(block);

        let mut local_variables: [Option<BasicValueEnum<'static>>; 50] = [None; 50];
        let mut stack: Vec<BasicValueEnum<'static>> = Vec::new();

        let arr_header_size = usize_type.const_int(mem::size_of::<usize>() as u64, false);
        let indexed_ptr = |array: PointerValue<'static>, index: IntValue<'static>, ty: LlvmReturnType<'static>| {
            let offset = self.builder.build_int_add(arr_header_size, self.builder.build_int_mul(ty.size_of().unwrap(), index, "ptrcalc interm"), "index offset");
            unsafe {
                self.builder.build_gep(self.context.i8_type(), array, &[offset], "indexed ptr")
            }
        };


        for instr in method.data.code.code.iter(..) {
            match instr {
                Instruction::IConst(x) => {
                    stack.push(self.context.i32_type().const_int(x as u64, false).into());
                }
                Instruction::IAdd => {
                    let a = stack.pop().unwrap().into_int_value();
                    let b = stack.pop().unwrap().into_int_value();
                    stack.push(self.builder.build_int_add(a, b, "result").into());
                }
                Instruction::AStore(i) => {
                    local_variables[i as usize] = Some(stack.pop().unwrap());
                }
                Instruction::ALoad(i) => {
                    stack.push(local_variables[i as usize].unwrap());
                }
                Instruction::NewArray(atype) => {
                    let ty = match atype {
                        4 => DescriptorEntry::Boolean,
                        5 => DescriptorEntry::Char,
                        6 => DescriptorEntry::Float,
                        7 => DescriptorEntry::Double,
                        8 => DescriptorEntry::Byte,
                        9 => DescriptorEntry::Short,
                        10 => DescriptorEntry::Int,
                        11 => DescriptorEntry::Long,
                        _ => panic!()
                    };

                    // malloc(sizeof(ty) * i + ARR_HEADER_SIZE);
                    let malloc_size = self.builder.build_int_add(self.builder.build_int_mul(stack.pop().unwrap().into_int_value(), ty.to_type(self.context).unwrap().size_of().unwrap(), "amalloc intermediate"), arr_header_size, "amalloc size");
                    stack.push(self.builder.build_array_malloc(self.context.i8_type(), malloc_size, "arrayptr").unwrap().into());
                }
                Instruction::IAstore => {
                    let ty = DescriptorEntry::Int.to_type(self.context).unwrap();
                    let value: IntValue<'static> = stack.pop().unwrap().into_int_value();
                    let index: IntValue<'static> = stack.pop().unwrap().into_int_value();
                    let array: PointerValue<'static> = stack.pop().unwrap().into_pointer_value();
                    self.builder.build_store(indexed_ptr(array, index, ty), value);
                }
                Instruction::IALoad => {
                    let ty = DescriptorEntry::Int.to_type(self.context).unwrap();
                    let index: IntValue<'static> = stack.pop().unwrap().into_int_value();
                    let array: PointerValue<'static> = stack.pop().unwrap().into_pointer_value();
                    stack.push(self.builder.build_load(ty.to_basic().unwrap(), indexed_ptr(array, index, ty), "iaload result"));
                }
                Instruction::IReturn => {
                    self.builder.build_return(Some(&stack.pop().unwrap()));
                }
                Instruction::IfICmpEq(i) => {
                    // self.builder.build_conditional_branch(comparison, then_block, else_block);
                }
                x => panic!("No LLVM implementation for {:?}", x),
            }
        }

        println!("Running {}", function.print_to_string());
        unsafe {
            let fun: JitFunction<unsafe extern "C" fn() -> i32> = self.execution_engine.get_function(&format!("{}-{}", class.name, method.data.name)).unwrap();
            let i = fun.call();
            println!("Result is {}", i);
        }
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

impl TryFrom<ClassFile> for LlvmClass {
    type Error = ();

    fn try_from(classfile: ClassFile) -> Result<Self, Self::Error> {
        let constant_pool = classfile.constant_pool;
        let this_class = constant_pool.get_as::<types::Class>(classfile.this_class).ok_or(())?;
        let fullname = constant_pool.get_as_string(this_class.name_index).ok_or(())?.to_string();
        let name = fullname.rsplit_once('/').unwrap_or(("", &fullname));
        
        let methods = classfile.methods.into_iter().map(|m| LlvmMethod::from_info(m, &constant_pool).unwrap()).collect(); // FIXME something better than unwrap pls

        Ok(LlvmClass {
            constant_pool,
            package: name.0.to_string(),
            name: name.1.to_string(),
            methods
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
