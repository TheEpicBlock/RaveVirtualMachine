use std::string::FromUtf8Error;
use thiserror::Error;
use crate::class_file::ClassFile;
use std::io::Read;
use crate::byte_util::ByteParseable;

mod byte_util;
pub mod class_file;
pub mod constant_pool;
pub mod attributes;
pub mod bytecode;

#[macro_use]
extern crate bitflags;

#[derive(Error, Debug)]
pub enum ClassParseError {
    #[error("whilst parsing method")]
    MethodParsingError(#[source] Box<ClassParseError>),
    #[error("whilst parsing attribute of type {0}")]
    AttributingError(String, #[source] Box<ClassParseError>),
    #[error("whilst {0}")]
    MiscContext(&'static str, #[source] Box<ClassParseError>),

    #[error("found wrong magic value: {0}")]
    WrongMagic(u32),
    #[error("found invalid constant table entry: {0}")]
    InvalidConstantTableEntry(u8),
    #[error("invalid bytecode: {0}")]
    InvalidBytecode(u8),
    #[error("Invalid constant pool index (is of wrong type or out of bounds): {0}")]
    InvalidConstantPoolIndex(u16),

    #[error("whilst parsing utf-8")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("io error ({0})")]
    IoError(std::io::Error),
}

impl From<std::io::Error> for ClassParseError {
    fn from(err: std::io::Error) -> Self {
        ClassParseError::IoError(err)
    }
}

impl ClassParseError {
    pub fn with_misc_context(self, ctx: &'static str) -> Self {
        ClassParseError::MiscContext(ctx, Box::new(self))
    }
}

pub fn parse(bytes: &mut impl Read) -> Result<ClassFile, ClassParseError> {
    ClassFile::parse(bytes)
}