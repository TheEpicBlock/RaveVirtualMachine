use vm_core::ClassLoader;
use vm_core::VirtualMachine;
use vm_cranelift::CraneliftJitCompiler;

fn run(classpath: impl ClassLoader) {
    let mut vm = VirtualMachine::new(classpath, CraneliftJitCompiler::default());
    vm.run("Addition", "number", "()I").unwrap();
}