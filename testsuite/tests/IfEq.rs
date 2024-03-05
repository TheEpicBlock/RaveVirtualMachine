use vm_core::interop::JavaInt;
use vm_core::ClassLoader;
use vm_core::VirtualMachine;
use vm_llvm::LlvmJitCompiler;

type JavaFunct = extern "C" fn() -> JavaInt;

fn run(classpath: impl ClassLoader) {
    let mut vm = VirtualMachine::new(classpath, LlvmJitCompiler::default());
    let func: JavaFunct = vm.get_fn_pointer("IfEq", "run").unwrap();
    
    assert_eq!(func(), 2);
}