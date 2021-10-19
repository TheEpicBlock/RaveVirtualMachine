use crate::byte_util::{read_to_vec, BigEndianReadExt, ByteParseable};
use crate::{gen_parseable, ClassParseError};
use std::io::{Read, Cursor};

pub trait ParseableWithCP {
    fn parse_bytes(bytes: &[u8], pool: &impl ConstantPool) -> Result<Self, ClassParseError> where Self: Sized {
        return Self::parse(&mut Cursor::new(bytes), pool);
    }

    fn parse(bytes: &mut impl Read, pool: &impl ConstantPool) -> Result<Self, ClassParseError> where Self: Sized;
}

impl<T: ByteParseable> ParseableWithCP for T {
    fn parse(bytes: &mut impl Read, _: &impl ConstantPool) -> Result<Self, ClassParseError> {
        Self::parse(bytes)
    }
}

pub fn parse_multiple_with_cp<T: ParseableWithCP>(bytes: &mut impl Read, pool: &impl ConstantPool, amount: usize) -> Result<Vec<T>, ClassParseError> {
    let mut result = Vec::with_capacity(amount);
    for _ in 0..amount {
        result.push(T::parse(bytes, pool)?);
    }
    return Ok(result);
}

macro_rules! gen_constant_pool {
    (
        $(#[$Meta:meta])*
        pub enum $Name:ident {
            $(
                $Type:ident($DataContainer:ident) = $Value:literal
            ),+
        }
    ) => {
        $(#[$Meta])*
        pub enum ConstantPoolEntry {
            $(
                $Type($DataContainer),
            )+
        }

        impl ByteParseable for ConstantPoolEntry {
            fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> {
                let tag = bytes.read_u8()?;
                match tag {
                    $(
                        $Value => Ok(Self::$Type(ByteParseable::parse(bytes)?)),
                    )+
                    _ => Err(ClassParseError::InvalidConstantTableEntry(tag))
                }
            }
        }

        pub mod types {
            use super::*;

            $(
                pub enum $Type {}

                impl ConstantPoolType for $Type {
                    type Inner = super::$DataContainer;

                    fn get_id() -> u8 {
                        $Value
                    }

                    fn inner_from_container(container: &ConstantPoolEntry) -> Option<&Self::Inner> {
                        match container {
                            ConstantPoolEntry::$Type(inner) => Some(inner),
                            _ => None,
                        }
                    }
                }
            )+
        }
    }
}

gen_constant_pool! {
    /// See: https://docs.oracle.com/javase/specs/jvms/se16/html/jvms-4.html#jvms-4.4.1
    #[derive(Debug, PartialEq)]
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
        PackageInfo(NameInfo) = 20
    }
}

gen_parseable! {
    #[derive(Debug, PartialEq)]
    pub struct NameInfo {
        name_index: u16,
    }

    #[derive(Debug, PartialEq)]
    pub struct TypeRefInfo {
        class_index: u16,
        name_and_type_index: u16,
    }

    #[derive(Debug, PartialEq)]
    pub struct StringInfo {
        string_index: u16,
    }

    #[derive(Debug, PartialEq)]
    pub struct NameAndTypeInfo {
        name_index: u16,
        descriptor_index: u16,
    }

    #[derive(Debug, PartialEq)]
    pub struct MethodHandleInfo {
        reference_kind: u8,
        reference_index: u16,
    }

    #[derive(Debug, PartialEq)]
    pub struct MethodTypeInfo {
        descriptor_index: u16,
    }

    #[derive(Debug, PartialEq)]
    pub struct DynamicInfo {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    }

    #[derive(Debug, PartialEq)]
    pub struct Integer{inner: u32,}
    #[derive(Debug, PartialEq)]
    pub struct Float{inner: f32,}
    #[derive(Debug, PartialEq)]
    pub struct Long{inner: u64,}
    #[derive(Debug, PartialEq)]
    pub struct Double{inner: f64,}
}

impl Integer {
    pub fn new(inner: u32) -> Self {
        Integer { inner }
    }
}

impl Long {
    pub fn new(inner: u64) -> Self {
        Long { inner }
    }
}

impl Float {
    pub fn new(inner: f32) -> Self {
        Float { inner }
    }
}

impl Double {
    pub fn new(inner: f64) -> Self {
        Double { inner }
    }
}

#[derive(Debug, PartialEq)]
pub struct Utf8Info{pub inner: String,}

impl ByteParseable for Utf8Info {
    fn parse(bytes: &mut impl Read) -> Result<Self, ClassParseError> where Self: Sized {
        let len = bytes.read_u16()?;
        let vec = read_to_vec(bytes, len as usize)?;
        // TODO doesn't respect custom string format
        Ok(Self {
            inner: String::from_utf8(vec)?
        })
    }
}

// Implemented on empty enums in the types crate
pub trait ConstantPoolType {
    type Inner;
    fn get_id() -> u8;
    fn inner_from_container(container: &ConstantPoolEntry) -> Option<&Self::Inner>;
}

pub trait ConstantPool {
    /// Returns the value at [`index`].
    /// This method is 0 indexed. It's recommended to use [`get_entry`] instead.
    ///
    /// # Examples
    /// ```
    /// use classfile_parser::constant_pool::{Float, Integer, ConstantPool};
    /// use classfile_parser::constant_pool::ConstantPoolEntry::{FloatInfo, IntegerInfo};
    ///
    /// let pool = vec![FloatInfo(Float::new(5f32)), IntegerInfo(Integer::new(9))];
    ///
    /// assert_eq!(pool.get_entry_0(0), Some(&FloatInfo(Float::new(5f32)))); // Type is implied
    /// ```
    fn get_entry_0(&self, index: u16) -> Option<&ConstantPoolEntry>;

    /// Returns the value at [`index`].
    /// This method is 1 indexed. See [`get_entry_0`] for a 0 indexed alternative.
    ///
    /// # Examples
    /// ```
    /// use classfile_parser::constant_pool::{Float, Integer, ConstantPool};
    /// use classfile_parser::constant_pool::ConstantPoolEntry::{FloatInfo, IntegerInfo};
    ///
    /// let pool = vec![FloatInfo(Float::new(5f32)), IntegerInfo(Integer::new(9))];
    ///
    /// assert_eq!(pool.get_entry(1), Some(&FloatInfo(Float::new(5f32)))); // Type is implied
    /// ```
    #[inline]
    fn get_entry(&self, index: u16) -> Option<&ConstantPoolEntry> {
        return self.get_entry_0(index-1);
    }

    /// Returns the value at [`index`] and unwraps it into the specified type.
    /// This method is 0 indexed. It's recommended to use [`get_as`].
    ///
    /// # Examples
    /// ```
    /// use classfile_parser::constant_pool::{Float, Integer, ConstantPool};
    /// use classfile_parser::constant_pool::ConstantPoolEntry::{FloatInfo, IntegerInfo};
    /// use classfile_parser::constant_pool::types;
    ///
    /// let pool = vec![FloatInfo(Float::new(5f32)), IntegerInfo(Integer::new(9))];
    ///
    /// assert_eq!(pool.get_as_0::<types::IntegerInfo>(1), Some(&Integer::new(9))); // Type is specified via turbofish
    ///
    /// // Will return None when the wrong type is present
    /// assert!(pool.get_as_0::<types::IntegerInfo>(0).is_none());
    ///
    /// // Will return None when the index is out of range
    /// assert!(pool.get_as_0::<types::IntegerInfo>(999).is_none());
    /// ```
    #[inline]
    fn get_as_0<T: ConstantPoolType>(&self, index: u16) -> Option<&T::Inner> {
        return T::inner_from_container(self.get_entry_0(index)?);
    }

    /// Returns the value at [`index`] and unwraps it into the specified type.
    /// This method is 1 indexed. See [`get_as_0`] for a 0 indexed alternative.
    ///
    /// # Examples
    /// ```
    /// use classfile_parser::constant_pool::{Float, Integer, ConstantPool};
    /// use classfile_parser::constant_pool::ConstantPoolEntry::{FloatInfo, IntegerInfo};
    /// use classfile_parser::constant_pool::types;
    ///
    /// let pool = vec![FloatInfo(Float::new(5f32)), IntegerInfo(Integer::new(9))];
    ///
    /// assert_eq!(pool.get_as::<types::IntegerInfo>(2), Some(&Integer::new(9))); // Type is specified via turbofish
    ///
    /// // Will return None when the wrong type is present
    /// assert!(pool.get_as::<types::IntegerInfo>(1).is_none());
    ///
    /// // Will return None when the index is out of range
    /// assert!(pool.get_as::<types::IntegerInfo>(999).is_none());
    /// ```
    #[inline]
    fn get_as<T: ConstantPoolType>(&self, index: u16) -> Option<&T::Inner> {
        return self.get_as_0::<T>(index-1);
    }

    /// Gets the total size of this pool
    ///
    /// # Examples
    /// ```
    /// use classfile_parser::constant_pool::{ConstantPoolEntry, Float, Integer, ConstantPool};
    /// use classfile_parser::constant_pool::ConstantPoolEntry::{FloatInfo, IntegerInfo};
    ///
    /// let pool = vec![FloatInfo(Float::new(5f32)), IntegerInfo(Integer::new(9))];
    ///
    /// assert_eq!(pool.size(), 2);
    /// ```
    fn size(&self) -> u16;
}

impl ConstantPool for Vec<ConstantPoolEntry> {
    #[inline]
    fn get_entry_0(&self, index: u16) -> Option<&ConstantPoolEntry> {
        return self.get(index as usize);
    }

    #[inline]
    fn size(&self) -> u16 {
        return self.len() as u16;
    }
}