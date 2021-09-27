use std::string::FromUtf8Error;
use thiserror::Error;

mod byte_util;
pub mod class_file;
pub mod constant_pool;
pub mod attributes;
mod bytecode;

#[macro_use]
extern crate bitflags;

#[derive(Error, Debug)]
pub enum ClassParseError {
    #[error("Wrong magic value found in class file: {0}")]
    WrongMagic(u32),
    #[error("Invalid constant table entry: {0}")]
    InvalidConstantTableEntry(u8),
    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(u8),
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Utf8 parsing error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Invalid constant pool index (is of wrong type or out of bounds): {0}")]
    InvalidConstantPoolIndex(u16),
}