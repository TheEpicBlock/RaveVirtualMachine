use std::iter::Map;
use std::collections::HashMap;
use classfile_parser::class_file::{ClassFile, MethodInfo, MethodAccessFlags};
use classfile_parser::constant_pool::ConstantPool;
use classfile_parser::constant_pool::types;
use bitflags::bitflags;
use crate::classfile_util::ConstantPoolExtensions;

pub(crate) struct ClassStore<'a> {
    classes: HashMap<String, Class<'a>>
}

impl<'a> Default for ClassStore<'a> {
    fn default() -> Self {
        todo!()
    }
}

impl<'a> ClassStore<'a> {
    pub fn add_from_classfile(&mut self, classfile: ClassFile) -> Result<&'a Class<'a>, ()> {
        let this_class = classfile.constant_pool.get_as::<types::Class>(classfile.this_class).ok_or(())?;
        let fullname = classfile.constant_pool.get_as_string(this_class.name_index).ok_or(())?.to_string();

        let name = fullname.rsplit_once("/").ok_or(())?;
        let class = Class {
            package: name.0,
            name: name.1,
            methods: vec![]
        };


        todo!()
    }
}

pub(crate) struct Class<'a> {
    package: &'a str,
    name: &'a str,
    methods: Vec<Method>,
}

impl<'a> Class<'a> {
    /// Locates a main method in this class if available
    pub fn find_main(&self) -> Option<&Method> {
        self.methods.iter().find(|method| method.is_main())
    }
}


bitflags! {
    pub struct MethodFlags: u8 {
        const STATIC = 0b00000001;
        const FINAL  = 0b00000010;
        const SYNCHRONISED = 0b00001000;
        // const BRIDGE = 0x0040;
        // const VARARGS = 0x0080;
        // const NATIVE = 0x0100;
        // const ABSTRACT = 0x0400;
        // const STRICT = 0x0800;
        // const SYNTHETIC = 0x1000;
    }
}

impl MethodFlags {
    pub fn from_access_flags(access_flags: &MethodAccessFlags) -> Self {
        let mut base = MethodFlags::empty();
        if access_flags.contains(MethodAccessFlags::STATIC) {
            base |= MethodFlags::STATIC;
        }
        if access_flags.contains(MethodAccessFlags::FINAL) {
            base |= MethodFlags::FINAL;
        }
        if access_flags.contains(MethodAccessFlags::SYNCHRONISED) {
            base |= MethodFlags::SYNCHRONISED;
        }
        return base;
    }
}

enum Visibility {
    Public,
    Protected,
    Private,
}

impl Visibility {
    pub fn from_flags(flags: &MethodAccessFlags) -> Self {
        if flags.contains(MethodAccessFlags::PUBLIC) {
            return Self::Public;
        } else if flags.contains(MethodAccessFlags::PROTECTED) {
            return Self::Protected;
        } else if flags.contains(MethodAccessFlags::PRIVATE) {
            return Self::Private;
        }
        Self::Private // Default to private
    }
}

pub(crate) struct Method {
    name: String,
    descriptor: String,
    visibility: Visibility,
    flags: MethodFlags,
}

impl Method {
    pub fn from_info(method_info: MethodInfo, constant_pool: impl ConstantPool) -> Result<Self, ()> {
        let name = constant_pool.get_as_string(method_info.name_index).ok_or(())?.to_string();
        let descriptor = constant_pool.get_as_string(method_info.descriptor).ok_or(())?.to_string();
        let visibility = Visibility::from_flags(&method_info.access_flags);
        let flags = MethodFlags::from_access_flags(&method_info.access_flags);
        todo!();
    }

    /// Checks if the function matches the definition of a main function
    pub fn is_main(&self) -> bool {
        self.name == "main" && self.descriptor == "([Ljava/lang/String;)V" && self.is_public() && self.is_static()
    }

    pub fn is_static(&self) -> bool {
        self.flags.contains(MethodFlags::STATIC)
    }

    pub fn is_public(&self) -> bool {
        matches!(self.visibility, Visibility::Public)
    }
}
