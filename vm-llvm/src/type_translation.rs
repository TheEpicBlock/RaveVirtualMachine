// Translates java types to llvm ones

use inkwell::{context::Context, types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, VoidType}, values::IntValue};
use vm_core::class_store::DescriptorEntry;

pub trait CtxJavaTypeExtension<'ctx> {
    fn java_byte(&'ctx self) -> IntType<'ctx>;
    fn java_char(&'ctx self) -> IntType<'ctx>;
    fn java_double(&'ctx self) -> FloatType<'ctx>;
    fn java_float(&'ctx self) -> FloatType<'ctx>;
    fn java_int(&'ctx self) -> IntType<'ctx>;
    fn java_long(&'ctx self) -> IntType<'ctx>;
    fn java_short(&'ctx self) -> IntType<'ctx>;
    fn java_bool(&'ctx self) -> IntType<'ctx>;
}

impl<'ctx> CtxJavaTypeExtension<'ctx> for Context {
    fn java_byte(&'ctx self) -> IntType<'ctx> {
        self.i8_type()
    }

    fn java_char(&'ctx self) -> IntType<'ctx> {
        self.i8_type()
    }

    fn java_double(&'ctx self) -> FloatType<'ctx> {
        todo!()
    }

    fn java_float(&'ctx self) -> FloatType<'ctx> {
        todo!()
    }

    fn java_int(&'ctx self) -> IntType<'ctx> {
        todo!()
    }

    fn java_long(&'ctx self) -> IntType<'ctx> {
        todo!()
    }

    fn java_short(&'ctx self) -> IntType<'ctx> {
        todo!()
    }

    fn java_bool(&'ctx self) -> IntType<'ctx> {
        todo!()
    }
}

pub trait IntoType {
    fn to_type<'ctx>(&self, ctx: &'ctx Context) -> Option<LlvmReturnType<'ctx>>;
    // fn to_abi(&self, target: TargetFrontendConfig) -> AbiParam {
    //     AbiParam::new(self.to_type(target))
    // }
}

impl IntoType for DescriptorEntry {
    fn to_type<'ctx>(&self, ctx: &'ctx Context) -> Option<LlvmReturnType<'ctx>> {
        // TODO ptr types shouldn't be to ints
        match self {
            // DescriptorEntry::Class(_) => ctx.i8_type().ptr_type(AddressSpace::default()).into(),
            DescriptorEntry::Byte =>        Some(BasicTypeEnum::from(ctx.java_byte()).into()),
            DescriptorEntry::Char =>        Some(BasicTypeEnum::from(ctx.java_char()).into()),
            DescriptorEntry::Double =>      Some(BasicTypeEnum::from(ctx.f64_type()).into()),
            DescriptorEntry::Float =>       Some(BasicTypeEnum::from(ctx.f32_type()).into()),
            DescriptorEntry::Int =>         Some(BasicTypeEnum::from(ctx.i32_type()).into()),
            DescriptorEntry::Long =>        Some(BasicTypeEnum::from(ctx.i64_type()).into()),
            DescriptorEntry::Short =>       Some(BasicTypeEnum::from(ctx.i16_type()).into()),
            DescriptorEntry::Boolean =>     Some(BasicTypeEnum::from(ctx.bool_type()).into()),
            DescriptorEntry::Void =>        Some(ctx.void_type().into()),
            // DescriptorEntry::Array(_) => ctx.i8_type().ptr_type(AddressSpace::default()).into(),
            _ => None
        }
    }
}

pub enum LlvmReturnType<'ctx> {
    Regular(BasicTypeEnum<'ctx>),
    Void(VoidType<'ctx>)
}

impl<'ctx> LlvmReturnType<'ctx> {
    pub fn to_meta(self) -> BasicMetadataTypeEnum<'ctx> {
        match self {
            LlvmReturnType::Regular(x) => x.into(),
            LlvmReturnType::Void(_x) => panic!(),
        }
    }

    pub fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        match self {
            LlvmReturnType::Regular(x) => x.fn_type(param_types, is_var_args),
            LlvmReturnType::Void(x) => x.fn_type(param_types, is_var_args),
        }
    }

    pub fn size_of(&self) -> Option<IntValue<'ctx>> {
        match self {
            LlvmReturnType::Regular(x) => x.size_of(),
            LlvmReturnType::Void(x) => None,
        }
    }

    pub fn to_basic(&self) -> Option<BasicTypeEnum<'ctx>> {
        match self {
            LlvmReturnType::Regular(x) => Some(*x),
            LlvmReturnType::Void(x) => None,
        }

    }
}

impl<'ctx> From<BasicTypeEnum<'ctx>> for LlvmReturnType<'ctx> {
    fn from(value: BasicTypeEnum<'ctx>) -> Self {
        Self::Regular(value)
    }
}

impl<'ctx> From<VoidType<'ctx>> for LlvmReturnType<'ctx> {
    fn from(value: VoidType<'ctx>) -> Self {
        Self::Void(value)
    }
}

pub trait LlvmType<'ctx> {
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx>;

    fn to_enum(self) -> BasicMetadataTypeEnum<'ctx>;
}

impl<'ctx> LlvmType<'ctx> for IntType<'ctx> {
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.fn_type(param_types, is_var_args)
    }

    fn to_enum(self) -> BasicMetadataTypeEnum<'ctx> {
        self.into()
    }
}

impl<'ctx> LlvmType<'ctx> for FloatType<'ctx> {
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.fn_type(param_types, is_var_args)
    }

    fn to_enum(self) -> BasicMetadataTypeEnum<'ctx> {
        self.into()
    }
}

impl<'ctx> LlvmType<'ctx> for VoidType<'ctx> {
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.fn_type(param_types, is_var_args)
    }

    fn to_enum(self) -> BasicMetadataTypeEnum<'ctx> {
        panic!("Void is not an argument");
    }
}