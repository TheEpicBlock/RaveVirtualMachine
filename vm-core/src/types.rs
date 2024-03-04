use std::convert::{TryFrom, TryInto};

use enum_map::Enum;

use crate::class_store::DescriptorEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum PrimitiveTypes {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Float,
    Long,
    Double,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum LvtEntryType {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Float,
    Reference,
    ReturnAddress
}

#[derive(Debug)]
pub struct IsReturnAddress;
#[derive(Debug)]
pub struct IsVoidType;

impl TryFrom<&LvtEntryType> for PrimitiveTypes {
    type Error = IsReturnAddress;

    fn try_from(value: &LvtEntryType) -> Result<PrimitiveTypes, Self::Error> {
        match value {
            LvtEntryType::Boolean => Ok(PrimitiveTypes::Boolean),
            LvtEntryType::Byte => Ok(PrimitiveTypes::Byte),
            LvtEntryType::Char => Ok(PrimitiveTypes::Char),
            LvtEntryType::Short => Ok(PrimitiveTypes::Short),
            LvtEntryType::Int => Ok(PrimitiveTypes::Int),
            LvtEntryType::Float => Ok(PrimitiveTypes::Float),
            LvtEntryType::Reference => Ok(PrimitiveTypes::Reference),
            LvtEntryType::ReturnAddress => Err(IsReturnAddress),
        }
    }
}

impl TryFrom<&DescriptorEntry> for PrimitiveTypes {
    type Error = IsVoidType;

    fn try_from(value: &DescriptorEntry) -> Result<Self, Self::Error> {
        match value {
            DescriptorEntry::Class(_) =>    Ok(PrimitiveTypes::Reference),
            DescriptorEntry::Byte =>        Ok(PrimitiveTypes::Byte),
            DescriptorEntry::Char =>        Ok(PrimitiveTypes::Char),
            DescriptorEntry::Double =>      Ok(PrimitiveTypes::Double),
            DescriptorEntry::Float =>       Ok(PrimitiveTypes::Float),
            DescriptorEntry::Int =>         Ok(PrimitiveTypes::Int),
            DescriptorEntry::Long =>        Ok(PrimitiveTypes::Long),
            DescriptorEntry::Short =>       Ok(PrimitiveTypes::Short),
            DescriptorEntry::Boolean =>     Ok(PrimitiveTypes::Boolean),
            DescriptorEntry::Array(_) =>    Ok(PrimitiveTypes::Reference),
            DescriptorEntry::Void =>        Err(IsVoidType),
        }
    }
}