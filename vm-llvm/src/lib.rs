use std::collections::HashMap;


use inkwell::types::{AnyType, AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, VoidType};
use inkwell::values::{AnyValue, BasicValueEnum};
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
use classfile_parser::class_file::{ClassFile, MethodInfo};

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
        
        let return_types: Vec<_> = desc.0.iter().map(|t| t.to_type(self.context).unwrap().to_meta()).collect();
        let ty = desc.1.to_type(self.context).unwrap().fn_type(&return_types, false);
        let function = self.module.add_function(&format!("{}-{}", class.name, method.data.name), ty, None);

        let block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(block);

        let mut stack: Vec<BasicValueEnum> = Vec::new();

        for instr in &method.data.code.code {
            match instr {
                Instruction::IConst(x) => {
                    stack.push(self.context.i32_type().const_int(*x as u64, false).into());
                }
                Instruction::IAdd => {
                    let a = stack.pop().unwrap().into_int_value();
                    let b = stack.pop().unwrap().into_int_value();
                    stack.push(self.builder.build_int_add(a, b, "result").into())
                }
                Instruction::IReturn => {
                    self.builder.build_return(Some(&stack.pop().unwrap()));
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

trait IntoType {
    fn to_type<'ctx>(&self, ctx: &'ctx Context) -> Option<LlvmReturnType<'ctx>>;
    // fn to_abi(&self, target: TargetFrontendConfig) -> AbiParam {
    //     AbiParam::new(self.to_type(target))
    // }
}

impl IntoType for DescriptorEntry {
    fn to_type<'ctx>(&self, ctx: &'ctx Context) -> Option<LlvmReturnType<'ctx>> {
        // TODO ptr types shouldn't be to ints
        match self {
            // DescriptorEntry::Class(_) => ctx.i8_type().ptr_type(AddressSpace::default()).into(),
            DescriptorEntry::Byte =>        Some(BasicTypeEnum::from(ctx.i8_type()).into()),
            DescriptorEntry::Char =>        Some(BasicTypeEnum::from(ctx.i8_type()).into()),
            DescriptorEntry::Double =>      Some(BasicTypeEnum::from(ctx.f64_type()).into()),
            DescriptorEntry::Float =>       Some(BasicTypeEnum::from(ctx.f32_type()).into()),
            DescriptorEntry::Int =>         Some(BasicTypeEnum::from(ctx.i32_type()).into()),
            DescriptorEntry::Long =>        Some(BasicTypeEnum::from(ctx.i64_type()).into()),
            DescriptorEntry::Short =>       Some(BasicTypeEnum::from(ctx.i16_type()).into()),
            DescriptorEntry::Boolean =>     Some(BasicTypeEnum::from(ctx.bool_type()).into()),
            DescriptorEntry::Void =>        Some(ctx.void_type().into()),
            // DescriptorEntry::Array(_) => ctx.i8_type().ptr_type(AddressSpace::default()).into(),
            _ => None
        }
    }
}

enum LlvmReturnType<'ctx> {
    Regular(BasicTypeEnum<'ctx>),
    Void(VoidType<'ctx>)
}

impl<'ctx> LlvmReturnType<'ctx> {
    fn to_meta(self) -> BasicMetadataTypeEnum<'ctx> {
        match self {
            LlvmReturnType::Regular(x) => x.into(),
            LlvmReturnType::Void(x) => panic!(),
        }
    }
}

impl<'ctx> LlvmReturnType<'ctx> {
    fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        match self {
            LlvmReturnType::Regular(x) => x.fn_type(param_types, is_var_args),
            LlvmReturnType::Void(x) => x.fn_type(param_types, is_var_args),
        }
    }
}

impl<'ctx> From<BasicTypeEnum<'ctx>> for LlvmReturnType<'ctx> {
    fn from(value: BasicTypeEnum<'ctx>) -> Self {
        Self::Regular(value)
    }
}

impl<'ctx> From<VoidType<'ctx>> for LlvmReturnType<'ctx> {
    fn from(value: VoidType<'ctx>) -> Self {
        Self::Void(value)
    }
}

trait LlvmType<'ctx> {
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx>;

    fn to_enum(self) -> BasicMetadataTypeEnum<'ctx>;
}

impl<'ctx> LlvmType<'ctx> for IntType<'ctx> {
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.fn_type(param_types, is_var_args)
    }

    fn to_enum(self) -> BasicMetadataTypeEnum<'ctx> {
        self.into()
    }
}

impl<'ctx> LlvmType<'ctx> for FloatType<'ctx> {
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.fn_type(param_types, is_var_args)
    }

    fn to_enum(self) -> BasicMetadataTypeEnum<'ctx> {
        self.into()
    }
}

impl<'ctx> LlvmType<'ctx> for VoidType<'ctx> {
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.fn_type(param_types, is_var_args)
    }

    fn to_enum(self) -> BasicMetadataTypeEnum<'ctx> {
        panic!("Void is not an argument");
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
