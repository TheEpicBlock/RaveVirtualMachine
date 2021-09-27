use core::fmt;
use std::error::Error;
use std::io::{Cursor, Read};
use std::iter::Zip;

use crate::byte_util::{BigEndianReadExt, ByteParseable};
use crate::class_file::parsing::{ClassParseError, ParsedClass};
use crate::class_file::attributing::{AttributedClass, AttributingError};
use std::convert::TryFrom;

pub mod attributing;
pub(crate) mod constant_pool;
pub mod parsing;
pub mod bytecode;

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

pub fn attribute(class: ParsedClass) -> Result<AttributedClass, AttributingError> {
    AttributedClass::try_from(class)
}