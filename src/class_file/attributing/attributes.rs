use crate::class_file::parsing::AttributeInfo;
use crate::class_file::attributing::{AttributingError, TryAttributeFrom};
use crate::class_file::constant_pool::ConstantPool;
use crate::byte_util::{ByteParseable, BigEndianReadExt, read_to_vec};
use crate::gen_parseable;
use std::io::Read;
use crate::class_file::bytecode::Instruction;

pub type Err = AttributingError;

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
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<Instruction>,
    pub exception_table: Vec<u8>,
    pub attributes: Vec<AttributeInfo>,
}

impl ByteParseable<Err> for CodeAttribute {
    fn parse(bytes: &mut impl Read) -> Result<Self, Err> where Self: Sized {
        let max_stack = bytes.read_u16()?;
        let max_locals = bytes.read_u16()?;

        let bytecode_size = bytes.read_u32()?;
        let mut bytecode_bytes = bytes.take(bytecode_size as u64);

        let mut bytecode = Vec::new();
        while bytecode_bytes.limit() != 0 {
            bytecode.push(ByteParseable::parse(&mut bytecode_bytes)?);
        }

        let exception_table_size = bytes.read_u16()?;
        let exception_table = read_to_vec(bytes, (exception_table_size * 8) as usize)?;

        let attributes_size = bytes.read_u16()? as usize;
        let attributes = AttributeInfo::parse_array(bytes, attributes_size)?;

        return Ok(CodeAttribute {
            max_stack,
            max_locals,
            code: bytecode,
            exception_table,
            attributes
        });
    }
}