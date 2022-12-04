use std::collections::HashMap;

use vm_core::{VirtualMachine, JitCompiler, ClassShell};
use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::{DataContext, Module};
use cranelift::codegen;
use cranelift::prelude::{FunctionBuilderContext, AbiParam, FunctionBuilder};
use classfile_parser::class_file::{FieldInfo, MethodInfo, ClassFile};
use vm_core::class_store::{MethodData};
use classfile_parser::bytecode::Instruction;
use classfile_parser::constant_pool::{ConstantPool, types, ConstantPoolEntry};
use classfile_parser::constant_pool::types::FieldRef;
use vm_core::classfile_util::ConstantPoolExtensions;

pub struct CraneliftJitCompiler {
    /// The function builder context, which is reused across multiple
    /// FunctionBuilder instances.
    builder_context: FunctionBuilderContext,

    /// The main Cranelift context, which holds the state for codegen. Cranelift
    /// separates this from `Module` to allow for parallel compilation, with a
    /// context per thread, though this isn't in the simple demo here.
    ctx: codegen::Context,

    /// The data context, which is to data objects what `ctx` is to functions.
    data_ctx: DataContext,

    /// The module, with the jit backend, which manages the JIT'd
    /// functions.
    module: JITModule,

    namesToIds: HashMap<String, ClassId>,
    classes: Vec<CraneliftClass>,
}

impl Default for CraneliftJitCompiler {
    fn default() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
            namesToIds: HashMap::new(),
            classes: vec![],
        }
    }
}

impl JitCompiler for CraneliftJitCompiler {
    type ClassId = ClassId;
    type MethodId = MethodId;
    type ClassShell = CraneliftClass;

    fn load(&mut self, classfile: classfile_parser::class_file::ClassFile) -> Result<ClassId,()> {
        let constant_pool = &classfile.constant_pool;
        let this_class = constant_pool.get_as::<types::Class>(classfile.this_class).ok_or(())?;
        let fullname = constant_pool.get_as_string(this_class.name_index).ok_or(())?.to_string();
        
        self.classes.push(CraneliftClass::try_from(classfile)?);
        let id = ClassId(self.classes.len());
        self.namesToIds.insert(fullname, id);

        return Ok(id);
    }

    fn get(&self, name: &str) -> Result<&Self::ClassShell,()> {
        Ok(&self.classes[self.namesToIds.get(name).ok_or(())?.0])
    }

    fn run(&mut self, class: ClassId, method: Self::MethodId) {
        let class = &self.classes[class.0];
        let method = &class.methods[method.0];

        let code = &method.data.code.code;
        self.ctx.clear();
        let constant_pool = &class.constant_pool;

        for inst in code {
            match inst {
                Instruction::GetStatic(x) => {
                    let field = constant_pool.get_as::<types::FieldRef>(*x).unwrap(); // FIXME
                    let class = constant_pool.get_as::<types::Class>(field.class_index).unwrap();
                    let name_and_type = constant_pool.get_as::<types::NameAndTypeInfo>(field.name_and_type_index).unwrap();

                    let class_name = constant_pool.get_as_string(class.name_index).unwrap();
                    let field_name = constant_pool.get_as_string(name_and_type.name_index).unwrap();
                    let field_desc = constant_pool.get_as_string(name_and_type.descriptor_index).unwrap();

                    println!("GetStatic: {class_name}#{field_name}{field_desc}");
                },
                _ => {

                }
            }
        }
    }
}

pub struct CraneliftClass {
    constant_pool: Vec<ConstantPoolEntry>,
    package: String,
    name: String,
    methods: Vec<CraneliftMethod>,
}

#[derive(Clone, Copy)]
pub struct ClassId(usize);

#[derive(Clone, Copy)]
pub struct MethodId(usize);

pub struct CraneliftMethod {
    pub data: MethodData
}

impl<'a> ClassShell for CraneliftClass {
    type Method = MethodId;

    fn find_main(&self) -> Option<Self::Method> {
        let methodIndex = self.methods.iter().enumerate().find(|m| m.1.data.is_main())?.0;
        Some(MethodId(methodIndex))
    }
}

impl TryFrom<ClassFile> for CraneliftClass {
    type Error = ();

    fn try_from(classfile: ClassFile) -> Result<Self, Self::Error> {
        let constant_pool = classfile.constant_pool;
        let this_class = constant_pool.get_as::<types::Class>(classfile.this_class).ok_or(())?;
        let fullname = constant_pool.get_as_string(this_class.name_index).ok_or(())?.to_string();
        let name = fullname.rsplit_once("/").ok_or(())?;
        
        let methods = classfile.methods.into_iter().map(|m| CraneliftMethod::from_info(m, &constant_pool).unwrap()).collect(); // FIXME something better than unwrap pls

        Ok(CraneliftClass {
            constant_pool: constant_pool,
            package: name.0.to_string(),
            name: name.1.to_string(),
            methods: methods
        })
    }
}

impl CraneliftMethod {
    fn from_info(info: MethodInfo, constant_pool: &impl ConstantPool) -> Result<Self, ()> {
        Ok(CraneliftMethod {  
            data: MethodData::from_info(info, constant_pool)?
        })
    }
}

#[cfg(test)]
mod tests {
    use cranelift::codegen;
    use cranelift_jit::{JITModule, JITBuilder};
    use cranelift_module::{DataContext, Linkage, Module};
    use cranelift::prelude::{FunctionBuilder, FunctionBuilderContext, AbiParam, Variable, EntityRef, InstBuilder};
    use core::mem;

    #[test]
    fn basic_compile() {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let mut module = JITModule::new(builder);

        let mut ctx = module.make_context();
        let mut function_builder_ctx = FunctionBuilderContext::new();
        let mut data_ctx = DataContext::new();

        //---0.69.0
        let int = module.target_config().pointer_type();

        // Function takes in 2 ints
        ctx.func.signature.params.push(AbiParam::new(int));
        ctx.func.signature.params.push(AbiParam::new(int));

        // Returns an int too
        ctx.func.signature.returns.push(AbiParam::new(int));

        let mut function_builder = FunctionBuilder::new(&mut ctx.func, &mut function_builder_ctx);
        let entry_block = function_builder.create_block();
        function_builder.append_block_params_for_function_params(entry_block);
        function_builder.switch_to_block(entry_block);
        function_builder.seal_block(entry_block);

        // Set up vars
        function_builder.block_params(entry_block);
        let inp1 = Variable::new(0); function_builder.declare_var(inp1, int); // Each variable has a global index. We also declare the type
        let inp2 = Variable::new(1); function_builder.declare_var(inp2, int);
        let ret  = Variable::new(2); function_builder.declare_var(ret , int);

        function_builder.def_var(inp1, function_builder.block_params(entry_block)[0]); // Set the variables to their inputs
        function_builder.def_var(inp2, function_builder.block_params(entry_block)[1]);

        let inp1_use = function_builder.use_var(inp1);
        let inp2_use = function_builder.use_var(inp2);
        let ins = function_builder.ins().iadd(inp1_use, inp2_use);
        function_builder.def_var(ret, ins);
        let ret_use = function_builder.use_var(ret);
        function_builder.ins().return_(&[ret_use]);

        function_builder.finalize();
        println!("{}", ctx.func.display(None));

        let id = module.declare_function("test_basic", Linkage::Export, &ctx.func.signature).map_err(|e| e.to_string()).unwrap();

        module.define_function(id, &mut ctx, &mut codegen::binemit::NullTrapSink{}, &mut codegen::binemit::NullStackMapSink{}).map_err(|e| e.to_string()).unwrap();
        module.clear_context(&mut ctx);
        module.finalize_definitions();
        let code = module.get_finalized_function(id);

        let code_fn;
        unsafe {
            code_fn = mem::transmute::<_, fn(usize, usize) -> usize>(code);
        }
        let result = code_fn(1,5);
        assert_eq!(result, 6);
    }
}
