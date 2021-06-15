use core::fmt;
use std::error::Error;
use std::io::{Cursor, Read};
use std::iter::Zip;

use crate::byte_util::{BigEndianReadExt, ByteParseable};
use crate::class_file::parsing::{ClassParseError, ParsedClass};

mod attributing;
mod constant_pool;
mod parsing;

/// A class goes through multiple stages before being used. This enum keeps track of them
pub enum Stage {
    /// The raw bytes are now parsed into data structures
    Parsed,
    /// Bytecode and attributes are parsed, together with bitflags
    Attributed,
}

pub trait BasicClass {
    fn get_stage() -> Stage;
}

pub fn parse(bytes: &mut impl Read) -> Result<ParsedClass, ClassParseError> {
    ParsedClass::parse(bytes)
}
