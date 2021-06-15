use core::fmt;
use std::error::Error;
use std::io::{Cursor, Read};
use std::iter::Zip;

use crate::byte_util::{BigEndianReadExt, ByteParseable};
use crate::class_file::parsing::{ParsedClass, ClassParseError};

mod parsing;
mod constantpool;

/// A class goes through multiple stages before being used. This enum keeps track of them
pub enum Stage {
    Parsing
}

pub fn parse(bytes: &mut impl Read) -> Result<ParsedClass, ClassParseError> {
    ParsedClass::parse(bytes)
}