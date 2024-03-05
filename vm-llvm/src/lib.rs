mod type_translation;

use std::collections::HashMap;
use std::mem;

use inkwell::basic_block::BasicBlock;
use enum_map::EnumMap;
use inkwell::passes::{PassBuilderOptions, PassManagerSubType};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetData, TargetMachine};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, VoidType};
use inkwell::values::{AnyValue, BasicValueEnum, IntValue, PointerValue};
use type_translation::{IntoBasicType, LlvmReturnType};
use vm_core::types::{IsReturnAddress, LvtEntryType, PrimitiveTypes};
use vm_core::{JitCompiler, ClassShell};
use vm_core::class_store::{DescriptorEntry, MethodData};
use vm_core::classfile_util::{split_code_into_basic_blocks, ConstantPoolExtensions};
use classfile_parser::constant_pool::{ConstantPool, types, ConstantPoolEntry};
use classfile_parser::bytecode::Instruction;
use inkwell::builder::{self, Builder};
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::{IntPredicate, OptimizationLevel};
use classfile_parser::class_file::{ClassFile, MethodInfo};

use crate::type_translation::{CtxJavaTypeExtension, IntoType};

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
        let e = m.create_jit_execution_engine(OptimizationLevel::Aggressive).unwrap();
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

fn run_passes_on(module: &Module, machine: &TargetData) {
    Target::initialize_all(&InitializationConfig::default());
    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .unwrap();

    module
        .run_passes("default<O3>", &target_machine, PassBuilderOptions::create())
        .unwrap();
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

    fn run<'cctx>(&'cctx mut self, class: ClassId, method: Self::MethodId) {
        // Retrieve some variables
        let class = &self.classes[class.0];
        let method = &class.methods[method.0];
        let desc = method.data.parse_descriptors();
        
        // Setup some LLVM stuff
        let usize_type = self.context.ptr_sized_int_type(self.execution_engine.get_target_data(), None);
        let return_types: Vec<_> = desc.0.iter().map(|t| t.to_type(self.context).to_meta()).collect();
        let ty = desc.1.to_type(self.context).fn_type(&return_types, false);
        let function = self.module.add_function(&format!("{}-{}", class.name, method.data.name), ty, None);

        // Split into basic blocks
        let entry_block = self.context.append_basic_block(function, "entry-init");
        let basic_blocks = split_code_into_basic_blocks(&method.data.code.code).into_iter()
            .map(|block_range| {
                (block_range.start, (block_range, self.context.append_basic_block(function, "")))
            })
            .collect::<HashMap<_,_>>();

        let local_variables: [LocalVariableEntry; 50] = [LocalVariableEntry::default(); 50];
        let stack: Vec<BasicValueEnum<'static>> = Vec::new();
        let mut cctx = CompilingContext { entry_block, context: self.context, builder: &self.builder, local_variables, stack };

        let arr_header_size = usize_type.const_int(mem::size_of::<usize>() as u64, false);
        let indexed_ptr = |array: PointerValue<'static>, index: IntValue<'static>, ty: LlvmReturnType<'static>| {
            let offset = self.builder.build_int_add(arr_header_size, self.builder.build_int_mul(ty.size_of().unwrap(), index, "ptrcalc interm"), "index offset");
            unsafe {
                self.builder.build_gep(self.context.i8_type(), array, &[offset], "indexed ptr")
            }
        };

        // let lvt_store = |index: u8, ty: LvtEntryType| {
        //     let ptr = local_variables[index as usize].get(self, ty);
        //     self.builder.build_store(*ptr, stack.pop().unwrap());
        // };

        fn lvt_store<'ctx, 'cctx>(cctx: &mut CompilingContext<'ctx, 'cctx>, ctx: &LlvmJitCompiler, index: u8, ty: LvtEntryType) {
                let ptr = cctx.local_variables[index as usize].get(cctx.entry_block, ctx, ty).clone();
                ctx.builder.build_store(ptr, cctx.stack.pop().unwrap());
        }

        fn lvt_load<'ctx, 'cctx>(cctx: &mut CompilingContext<'ctx, 'cctx>, ctx: &LlvmJitCompiler, index: u8, ty: LvtEntryType) {
            let ptr = cctx.local_variables[index as usize].get(cctx.entry_block, ctx, ty).clone();
            cctx.stack.push(ctx.builder.build_load(ty.to_basic_type(ctx.context), ptr, ""));
        }

        for (block_bytes, block) in basic_blocks.values() {
            self.builder.position_at_end(*block);
            let mut ended_with_branch = false;
            for (byte, instr) in method.data.code.code.iter(block_bytes.clone()) {
                ended_with_branch = false;
                match instr {
                    Instruction::IConst(x) => {
                        cctx.stack.push(self.context.i32_type().const_int(x as u64, false).into());
                    }
                    Instruction::SIPush(short) => {
                        cctx.stack.push(self.context.java_int().const_int(short as u64, true).into());
                    }
                    Instruction::IAdd => {
                        let a = cctx.stack.pop().unwrap().into_int_value();
                        let b = cctx.stack.pop().unwrap().into_int_value();
                        cctx.stack.push(self.builder.build_int_add(a, b, "result").into());
                    },
                    Instruction::IInc(local_variable, constant) => {
                        let aa = &mut cctx;
                        let ptr = aa.local_variables[local_variable as usize].get(entry_block, &self, LvtEntryType::Int);
                        let val = self.builder.build_load(LvtEntryType::Int.to_basic_type(self.context), *ptr, "iinc load");
                        let val = val.into_int_value(); // Should never panic
                        let val = self.builder.build_int_add(val, self.context.java_int().const_int(bytemuck::cast(constant as u64), true), "");
                        self.builder.build_store(*ptr, val);
                    },
                    Instruction::AStore(i) => lvt_store(&mut cctx, &self, i, LvtEntryType::Reference),
                    Instruction::FStore(i) => lvt_store(&mut cctx, &self, i, LvtEntryType::Float),
                    Instruction::IStore(i) => lvt_store(&mut cctx, &self, i, LvtEntryType::Int),
                    Instruction::ALoad(i) => lvt_load(&mut cctx, &self, i, LvtEntryType::Reference),
                    Instruction::FLoad(i) => lvt_load(&mut cctx, &self, i, LvtEntryType::Float),
                    Instruction::ILoad(i) => lvt_load(&mut cctx, &self, i, LvtEntryType::Int),
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
                        let malloc_size = self.builder.build_int_add(self.builder.build_int_mul(cctx.stack.pop().unwrap().into_int_value(), ty.to_type(self.context).size_of().unwrap(), "amalloc intermediate"), arr_header_size, "amalloc size");
                        cctx.stack.push(self.builder.build_array_malloc(self.context.i8_type(), malloc_size, "arrayptr").unwrap().into());
                    }
                    Instruction::IAstore => {
                        let ty = DescriptorEntry::Int.to_type(self.context);
                        let value: IntValue<'static> = cctx.stack.pop().unwrap().into_int_value();
                        let index: IntValue<'static> = cctx.stack.pop().unwrap().into_int_value();
                        let array: PointerValue<'static> = cctx.stack.pop().unwrap().into_pointer_value();
                        self.builder.build_store(indexed_ptr(array, index, ty), value);
                    }
                    Instruction::IALoad => {
                        let ty = DescriptorEntry::Int.to_type(self.context);
                        let index: IntValue<'static> = cctx.stack.pop().unwrap().into_int_value();
                        let array: PointerValue<'static> = cctx.stack.pop().unwrap().into_pointer_value();
                        cctx.stack.push(self.builder.build_load(ty.to_basic().unwrap(), indexed_ptr(array, index, ty), "iaload result"));
                    }
                    Instruction::IReturn => {
                        self.builder.build_return(Some(&cctx.stack.pop().unwrap()));
                        ended_with_branch = true;
                    }
                    Instruction::IfEq(o) => {
                        let num = cctx.stack.pop().unwrap().into_int_value();
                        let comp = self.builder.build_int_compare(IntPredicate::EQ, num, self.context.java_int().const_zero().into(), "");
                        self.builder.build_conditional_branch(comp, basic_blocks[&((byte as i64 + o as i64) as usize)].1, basic_blocks[&(byte + instr.byte_size())].1);
                        ended_with_branch = true;
                    }
                    Instruction::Goto(o) => {
                        self.builder.build_unconditional_branch(basic_blocks[&((byte as i64 + o as i64) as usize)].1);
                        ended_with_branch = true;
                    }
                    x => panic!("No LLVM implementation for {:?}", x),
                }
            }
            if !ended_with_branch {
                if let Some(next_block) = basic_blocks.get(&block_bytes.end) {
                    self.builder.build_unconditional_branch(next_block.1);
                }
            }
        }
        // Branch to the first block from the init block
        self.builder.position_at_end(entry_block);
        self.builder.build_unconditional_branch(basic_blocks[&0].1);

        run_passes_on(&self.module, self.execution_engine.get_target_data());
        println!("Running {}", function.print_to_string());
        unsafe {
            let fun: JitFunction<unsafe extern "C" fn() -> i32> = self.execution_engine.get_function(&format!("{}-{}", class.name, method.data.name)).unwrap();
            let i = fun.call();
            println!("Result is {}", i);
        }
    }
}

pub struct CompilingContext<'ctx, 'cctx> {
    entry_block: BasicBlock<'ctx>,
    context: &'cctx Context,
    builder: &'cctx Builder<'ctx>,
    stack: Vec<BasicValueEnum<'ctx>>,
    local_variables: [LocalVariableEntry<'ctx>; 50],
}

#[derive(Default, Clone, Copy)]
pub struct LocalVariableEntry<'ctx> {
    allocs: EnumMap<LvtEntryType, Option<PointerValue<'ctx>>>,
}

impl<'ctx> LocalVariableEntry<'ctx> {
    pub fn get(&mut self, entry_block: BasicBlock<'ctx>, compiler: &LlvmJitCompiler, ty: LvtEntryType) -> &PointerValue<'ctx> {
        self.allocs[ty].get_or_insert_with(move || {
            let prev_block = compiler.builder.get_insert_block();
            compiler.builder.position_at_end(entry_block);

            let ret = compiler.builder.build_alloca(ty.to_basic_type(compiler.context), "");
            
            compiler.builder.position_at_end(prev_block.unwrap());
            return ret;
        })
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
