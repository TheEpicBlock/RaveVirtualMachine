use crate::class_file::parsing::MethodInfo;
use crate::vm::rcodes::RCode;
use crate::class_file::attributing::AttributedMethod;
use crate::class_file::attributing::attribute_parsing::ParsedAttribute;

pub struct Method {
    pub code: Vec<RCode>
}

impl Method {
    pub fn new(info: &AttributedMethod) -> Self {
        let mut parsed_code = None;
        for attribute in &info.attributes {
            if let ParsedAttribute::Code(code) = attribute {
                let mut code_buf = vec![];
                for instr in &code.code {
                    code_buf.push(RCode::from(instr));
                }
                parsed_code = Some(code_buf);
            }
        }
        Self {
            code: parsed_code.unwrap()
        }
    }
}