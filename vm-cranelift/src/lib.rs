use vm_core::{VirtualMachine, JitCompiler};
use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::{DataContext, Module};
use cranelift::codegen;
use cranelift::prelude::{FunctionBuilderContext, AbiParam, FunctionBuilder};
use classfile_parser::class_file::MethodInfo;
use vm_core::class_store::{ClassStore, Method};

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
        }
    }
}

impl JitCompiler for CraneliftJitCompiler {
    type MethodData = MethodJitData;

    fn compile(&mut self, method: &Method<Self::MethodData>, class_store: &ClassStore<Self::MethodData>) {
        let code = &method.code.code;
        self.ctx.clear();

        for inst in code {
            match inst {
                _ => {

                }
            }
        }
    }
}

pub struct MethodJitData {

}

impl Default for MethodJitData {
    fn default() -> Self {
        Self {
        }
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
