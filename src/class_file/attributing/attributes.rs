use crate::class_file::parsing::AttributeInfo;
use crate::class_file::attributing::AttributingError;
use crate::class_file::constant_pool::ConstantPool;
use crate::byte_util::ByteParseable;
use crate::gen_parseable;
use std::io::Read;

type Err = AttributingError;

pub trait AttributeParseable {
    fn parse(info: AttributeInfo, pool: impl ConstantPool) -> Result<Self, Err> where Self: Sized;
}

impl<T: ByteParseable<Err>> AttributeParseable for T {
    fn parse(info: AttributeInfo, pool: impl ConstantPool) -> Result<Self, Err> {
        Self::parse(&mut info.get_reader())
    }
}

pub trait Attribute: AttributeParseable{

}

gen_parseable! {
    const ERR = Err;

    pub struct ConstantValueAttribute {
        value_index: u16,
    }
}
