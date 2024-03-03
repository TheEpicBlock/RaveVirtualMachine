use std::collections::HashSet;
use std::ops::Range;

use classfile_parser::bytecode::Code;
use classfile_parser::bytecode::Instruction;
use classfile_parser::class_file::MethodInfo;
use classfile_parser::constant_pool::ConstantPool;
use classfile_parser::constant_pool::types;
use classfile_parser::attributes::{AttributeEntry, CodeAttribute};

pub trait ConstantPoolExtensions: ConstantPool {
    fn get_as_string(&self, index: u16) -> Option<&str> {
        self.get_as::<types::Utf8Info>(index).map(|v| v.inner.as_str())
    }
}

impl<R: ConstantPool + ?Sized> ConstantPoolExtensions for R {}

pub fn get_code_attribute(method: &MethodInfo) -> Option<&CodeAttribute> {
    for attribute in &method.attributes {
        if let AttributeEntry::Code(inner) = attribute {
            return Some(inner);
        }
    }
    None
}

/// Splits java bytecode into blocks, such that the only jumps
/// made by the bytecode are into the start of the blocks.
/// Returned is a list of byte-ranges into the bytecode. 
pub fn split_code_into_basic_blocks(code: &Code) -> Vec<Range<usize>> {
    let mut starting_positions = HashSet::new();

    starting_positions.insert(0);

    for (byte, inst) in code.iter(..) {
        match inst {
            Instruction::IfACmpEq(offset) |
            Instruction::IfACmpNe(offset) |
            Instruction::IfICmpEq(offset) |
            Instruction::IfICmpGe(offset) |
            Instruction::IfICmpGt(offset) |
            Instruction::IfICmpLe(offset) |
            Instruction::IfICmpLt(offset) |
            Instruction::IfICmpNe(offset) |
            Instruction::IfEq(offset) |
            Instruction::IfGe(offset) |
            Instruction::IfGt(offset) |
            Instruction::IfLe(offset) |
            Instruction::IfLt(offset) |
            Instruction::IfNe(offset) |
            Instruction::IfNonNull(offset) |
            Instruction::IfNull(offset) => {
                starting_positions.insert(byte + offset as usize);
                // If the jump is false we continue straight after
                starting_positions.insert(byte + inst.byte_size());
            }
            Instruction::Goto(offset) => {
                starting_positions.insert(byte + offset as usize);
            }
            Instruction::Goto_w(offset) => {
                starting_positions.insert(byte + offset as usize);
            }
            Instruction::JSr(offset) => {
                starting_positions.insert(byte + offset as usize);
                // The jsr instruction pushes the continuation address, a ret instruction can later return here
                starting_positions.insert(byte + inst.byte_size());
            }
            Instruction::JSr_w(offset) => {
                starting_positions.insert(byte + offset as usize);
                // The jsr instruction pushes the continuation address, a ret instruction can later return here
                starting_positions.insert(byte + inst.byte_size());
            }
            _ => {}
        }
    }

    let mut sorted: Vec<_> = starting_positions.iter().cloned().collect();
    sorted.sort();

    let mut ranges = Vec::with_capacity(sorted.len());
    for i in 0..sorted.len() {
        ranges.push(sorted[i]..*sorted.get(i+1).unwrap_or(&code.byte_len()));
    }
    return ranges;
}