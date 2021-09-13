use crate::byte_util::{read_to_vec, BigEndianReadExt, ByteParseable};
use crate::class_file::parsing::ClassParseError;
use crate::gen_parseable;
use std::io::Read;
use std::any::{Any, TypeId};

macro_rules! gen_constant_pool {
    (
        pub enum $Name:ident {
            $(
                $Flag:ident($Type:ident) = $Value:expr,
            )+
        }
    ) => {
        pub mod types {
            use super::*;
            pub trait ConstantPoolType {
                type Inner;
                fn get_index() -> u8;
                fn inner_from_container(container: &ConstantPoolEntry) -> Option<&Self::Inner>;
            }

            $(
                pub enum $Flag {}

                impl ConstantPoolType for $Flag {
                    type Inner = super::$Type;
                    fn get_index() -> u8 {
                        $Value
                    }

                    fn inner_from_container(container: &ConstantPoolEntry) -> Option<&Self::Inner> {
                        match container {
                            ConstantPoolEntry::$Flag(inner) => Some(inner),
                            _ => None,
                        }
                    }
                }
            )+
        }

        #[derive(Debug)]
        pub enum ConstantPoolEntry {
            $(
                $Flag($Type),
            )+
        }

        impl ByteParseable<ClassParseError> for ConstantPoolEntry {
            fn parse(mut bytes: &mut impl Read) -> Result<Self, ClassParseError> {
                let tag = bytes.read_u8()?;
                match tag {
                    $(
                        $Value => Ok(Self::$Flag(ByteParseable::parse(bytes)?)),
                    )+
                    _ => Err(ClassParseError::InvalidConstantTableEntry(tag))
                }
            }
        }
    }
}

/// See: https://docs.oracle.com/javase/specs/jvms/se16/html/jvms-4.html#jvms-4.4.1
gen_constant_pool! {
    pub enum ConstantPoolTypes {
        Class(NameInfo) = 7,
        FieldRef(TypeRefInfo) = 9,
        MethodRef(TypeRefInfo) = 10,
        InterfaceMethodRef(TypeRefInfo) = 11,
        StringInfo(StringInfo) = 8,
        IntegerInfo(Integer) = 3,
        FloatInfo(Float) = 4,
        LongInfo(Long) = 5,
        DoubleInfo(Double) = 6,
        NameAndTypeInfo(NameAndTypeInfo) = 12,
        Utf8Info(Utf8Info) = 1,
        MethodHandleInfo(MethodHandleInfo) = 15,
        MethodTypeInfo(MethodTypeInfo) = 16,
        DynamicInfo(DynamicInfo) = 17,
        InvokeDynamicInfo(DynamicInfo) = 18,
        ModuleInfo(NameInfo) = 19,
        PackageInfo(NameInfo) = 20,
    }
}

gen_parseable! {
    const ERR = ClassParseError;

    #[derive(Debug)]
    pub struct NameInfo {
        name_index: u16,
    }

    #[derive(Debug)]
    pub struct TypeRefInfo {
        class_index: u16,
        name_and_type_index: u16,
    }

    #[derive(Debug)]
    pub struct StringInfo {
        string_index: u16,
    }

    #[derive(Debug)]
    pub struct NameAndTypeInfo {
        name_index: u16,
        descriptor_index: u16,
    }

    #[derive(Debug)]
    pub struct MethodHandleInfo {
        reference_kind: u8,
        reference_index: u16,
    }

    #[derive(Debug)]
    pub struct MethodTypeInfo {
        descriptor_index: u16,
    }

    #[derive(Debug)]
    pub struct DynamicInfo {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    }

    #[derive(Debug)]
    pub struct Integer{inner: u32,}
    #[derive(Debug)]
    pub struct Float{inner: f32,}
    #[derive(Debug)]
    pub struct Long{inner: u64,}
    #[derive(Debug)]
    pub struct Double{inner: f64,}
}

#[derive(Debug)]
pub struct Utf8Info{pub inner: String,}

impl ByteParseable<ClassParseError> for Utf8Info {
    fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> where Self: Sized {
        let len = bytes.read_u16()?;
        let vec = read_to_vec(bytes, len as usize)?;
        // TODO doesn't respect custom string format
        Ok(Self {
            inner: String::from_utf8(vec)?
        })
    }
}

pub trait ConstantPool {
    fn get(&self, index: u16) -> Option<&ConstantPoolEntry>;

    fn get_as<T: types::ConstantPoolType>(&self, index: u16) -> Option<&T::Inner> {
        return T::inner_from_container(self.get(index)?);
    }
}

impl ConstantPool for Vec<ConstantPoolEntry> {
    fn get(&self, index: u16) -> Option<&ConstantPoolEntry> {
        if index < 1 || index as usize-1 > self.len() {
            return None;
        }

        return Some(&self[index as usize-1]);
    }
}