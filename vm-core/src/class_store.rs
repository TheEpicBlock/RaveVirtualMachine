use std::borrow::Borrow;
use std::iter::Map;
use std::collections::HashMap;
use classfile_parser::class_file::{ClassFile, MethodInfo, MethodAccessFlags};
use classfile_parser::constant_pool::ConstantPool;
use classfile_parser::constant_pool::types;
use bitflags::bitflags;
use classfile_parser::attributes::CodeAttribute;
use crate::classfile_util::ConstantPoolExtensions;

pub(crate) struct ClassStore {
    classes: HashMap<String, Class>,
}

impl Default for ClassStore {
    fn default() -> Self {
        ClassStore {
            classes: HashMap::new(),
        }
    }
}

impl ClassStore {
    pub fn add_from_classfile(&mut self, classfile: ClassFile) -> Result<&Class, ()> {
        let constant_pool = classfile.constant_pool;
        let this_class = constant_pool.get_as::<types::Class>(classfile.this_class).ok_or(())?;
        let fullname = constant_pool.get_as_string(this_class.name_index).ok_or(())?.to_string();

        let name = fullname.rsplit_once("/").ok_or(())?;
        let class = Class {
            package: name.0.to_string(),
            name: name.1.to_string(),
            methods: classfile.methods.into_iter().map(|m| Method::from_info(m, &constant_pool).unwrap()).collect() // FIXME something better than unwrap pls
        };

        let copy_of_fullname = fullname.clone();
        self.classes.insert(fullname, class);
        return Ok(&self.classes[&copy_of_fullname]);
    }
}

pub(crate) struct Class {
    package: String,
    name: String,
    methods: Vec<Method>,
}

impl Class {
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

pub(crate) enum Visibility {
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
    pub name: String,
    pub descriptor: String,
    pub visibility: Visibility,
    pub flags: MethodFlags,
    pub code: CodeAttribute,
}

impl Method {
    pub fn from_info(method_info: MethodInfo, constant_pool: &impl ConstantPool) -> Result<Self, ()> {
        let name = constant_pool.get_as_string(method_info.name_index).ok_or(())?.to_string();
        let descriptor = constant_pool.get_as_string(method_info.descriptor).ok_or(())?.to_string();
        let visibility = Visibility::from_flags(&method_info.access_flags);
        let flags = MethodFlags::from_access_flags(&method_info.access_flags);
        let mut code = Option::None;

        for attribute in method_info.attributes {
            if let classfile_parser::attributes::AttributeEntry::Code(code_attribute) = attribute {
                code = Some(code_attribute);
            }
        }

        Ok(Method {
            name,
            descriptor,
            visibility,
            flags,
            code: code.unwrap(),
        })
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
