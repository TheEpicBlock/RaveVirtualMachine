use vm_core::ClassLoader;
use vm_core::VirtualMachine;
use vm_llvm::LlvmJitCompiler;

fn run(classpath: impl ClassLoader) {
    let mut vm = VirtualMachine::new(classpath, LlvmJitCompiler::default());
    vm.run("IntArray", "run", "()I").unwrap();
}