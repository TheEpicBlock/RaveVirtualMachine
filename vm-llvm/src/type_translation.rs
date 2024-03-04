// Translates java types to llvm ones

use inkwell::{context::Context, types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, VoidType}, values::IntValue, AddressSpace};
use vm_core::{class_store::DescriptorEntry, types::{LvtEntryType, PrimitiveTypes}};

use crate::LocalVariableEntry;

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
        self.i32_type()
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
    fn to_type<'ctx>(&self, ctx: &'ctx Context) -> LlvmReturnType<'ctx>;
}

pub trait IntoBasicType {
    fn to_basic_type<'ctx>(&self, ctx: &'ctx Context) -> BasicTypeEnum<'ctx>;
}

impl IntoType for DescriptorEntry {
    fn to_type<'ctx>(&self, ctx: &'ctx Context) -> LlvmReturnType<'ctx> {
        let primative: Result<PrimitiveTypes, _> = self.try_into();
        primative.map_or(ctx.void_type().into(), |p| p.to_basic_type(ctx).into())
    }
}

impl IntoBasicType for LvtEntryType {
    fn to_basic_type<'ctx>(&self, ctx: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match &PrimitiveTypes::try_from(self) {
            Ok(ty) => ty.to_basic_type(ctx),
            Err(_) => todo!(),
        }
    }
}

impl IntoBasicType for PrimitiveTypes {
    fn to_basic_type<'ctx>(&self, ctx: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            PrimitiveTypes::Byte =>        BasicTypeEnum::from(ctx.java_byte()).into(),
            PrimitiveTypes::Char =>        BasicTypeEnum::from(ctx.java_char()).into(),
            PrimitiveTypes::Double =>      BasicTypeEnum::from(ctx.f64_type()).into(),
            PrimitiveTypes::Float =>       BasicTypeEnum::from(ctx.f32_type()).into(),
            PrimitiveTypes::Int =>         BasicTypeEnum::from(ctx.i32_type()).into(),
            PrimitiveTypes::Long =>        BasicTypeEnum::from(ctx.i64_type()).into(),
            PrimitiveTypes::Short =>       BasicTypeEnum::from(ctx.i16_type()).into(),
            PrimitiveTypes::Boolean =>     BasicTypeEnum::from(ctx.bool_type()).into(),
            PrimitiveTypes::Reference =>   BasicTypeEnum::from(ctx.i8_type().ptr_type(AddressSpace::default())).into(),
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