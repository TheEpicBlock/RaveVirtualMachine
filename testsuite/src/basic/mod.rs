use vm_core::{interop::JavaInt, VirtualMachine};
use vm_llvm::LlvmJitCompiler;

use crate::{include_class, setup_classloader};

// All tests of the basic type have a static method
// called `run` which returns an int.

type JavaBasicFunct = extern "C" fn() -> JavaInt;

fn basic_test(class: &[u8], class_name: &str, expected: JavaInt) {
    let classpath = setup_classloader(class);
    let mut vm = VirtualMachine::new(classpath, LlvmJitCompiler::default());
    let func: JavaBasicFunct = vm.get_fn_pointer(class_name, "run").unwrap();

    assert_eq!(func(), expected);
}

#[test]
fn addition() {
    basic_test(include_class!("/basic/Addition.class"), "Addition", 6);
}

#[test]
fn add_twice() {
    basic_test(include_class!("/basic/AddTwice.class"), "AddTwice", 9);
}

#[test]
fn if_eq() {
    basic_test(include_class!("/basic/IfEq.class"), "IfEq", 2);
}

#[test]
fn int_array() {
    basic_test(include_class!("/basic/IntArray.class"), "IntArray", 5);
}

#[test]
fn simple_return() {
    basic_test(include_class!("/basic/SimpleReturn.class"), "SimpleReturn", 5);
}

#[test]
fn simple_while() {
    basic_test(include_class!("/basic/SimpleWhile.class"), "SimpleWhile", 0);
}