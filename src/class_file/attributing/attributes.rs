use crate::class_file::parsing::AttributeInfo;
use crate::class_file::attributing::{AttributingError, TryAttributeFrom};
use crate::class_file::constant_pool::ConstantPool;
use crate::byte_util::ByteParseable;
use crate::gen_parseable;
use std::io::Read;

type Err = AttributingError;

impl<T: ByteParseable<Err>> TryAttributeFrom<AttributeInfo> for T {
    fn parse(info: AttributeInfo, pool: &impl ConstantPool) -> Result<Self, Err> {
        Self::parse(&mut info.get_reader())
    }
}

pub trait Attribute: TryAttributeFrom<AttributeInfo> {

}

gen_parseable! {
    const ERR = Err;

    pub struct ConstantValueAttribute {
        value_index: u16,
    }
}

pub struct CodeAttribute {
    max_stack: u16,
    max_locals: u16,
    code: Vec<()>,
    exception_table: Vec<()>,
    attributes: Vec<AttributeInfo>
}
//
// impl ByteParseable<Err> for CodeAttribute {
//     fn parse(bytes: &mut impl Read) -> Result<Self, Err> where Self: Sized {
//
//     }
// }
//
// fn parse_code(bytes: &mut impl Read) -> Vec<> {
//
// }