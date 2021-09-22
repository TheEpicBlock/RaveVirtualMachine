pub mod code_source;
mod rcodes;
mod method;
mod types;

use crate::class_file::parsing::ParsedClass;
use std::collections::HashMap;
use crate::vm::method::Method;
use crate::class_file::constant_pool::ConstantPool;
use crate::class_file::constant_pool::types::{ConstantPoolType, StringInfo, Utf8Info};
use crate::class_file::attributing::AttributedClass;
use crate::vm::rcodes::RCode;
use crate::vm::types::JavaType;

pub fn exec(code: AttributedClass) {
    let mut vm = VM::new(code);
    vm.exec();
}

pub struct VM {
    source: AttributedClass,
    compiled_methods: HashMap<String, Method>,
}

impl VM {
    pub fn new(code: AttributedClass) -> Self {
        Self {
            source: code,
            compiled_methods: HashMap::new(),
        }
    }

    pub fn get_method(&mut self, name: &str) -> Option<&Method> {
        if !self.compiled_methods.contains_key(name) {
            for method in &self.source.methods {
                if let Some(method_name) = &self.source.constant_pool.get_as::<Utf8Info>(method.name_index) {
                    if method_name.inner == name {
                        let compiled_method = Method::new(method);
                        self.compiled_methods.insert(name.to_owned(), compiled_method);
                    }
                }
            }
        }

        return self.compiled_methods.get(name);
    }

    pub fn exec(&mut self) {
        let a = self.get_method("<init>");
        let b = self.get_method("main");
        if let Some(init) =  a {
            self.exec_method(init);
        }
        //
        // if let Some(init) = self.get_method("main") {
        //     self.exec_method(init);
        // }
    }

    fn exec_method(&mut self, method: &Method) {
        for instr in &method.code {
            match instr {
                RCode::ArrayLoad(_) => {}
                RCode::ArrayStore(_) => {}
                RCode::PushConst(_) => {}
                RCode::LoadLocalVar(_, _) => {}
                RCode::StoreLocalVar(_, _) => {}
                RCode::GetField(_) => {}
                RCode::GetStaticField(_) => {}
                RCode::InvokeSpecial(_) => {}
                RCode::InvokeVirtual(_) => {}
                RCode::Return(expected_type) => {
                    if !matches!(expected_type, JavaType::Void) {

                    }
                }
                RCode::NOP => {}
            }
        }
    }
}