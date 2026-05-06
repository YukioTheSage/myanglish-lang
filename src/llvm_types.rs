/// LLVM type mapping placeholders.
///
/// Phase A note:
/// - This file keeps the API surface for future LLVM integration.
/// - Full type lowering is enabled only with the `llvm-backend` feature.

#[cfg(feature = "llvm-backend")]
mod llvm_enabled {
    use crate::ast::Type;
    use inkwell::context::Context;
    use inkwell::types::{BasicType, BasicTypeEnum, PointerType, StructType};

    /// Runtime string abstraction: { ptr: i8*, len: i64, cap: i64 }
    pub fn string_type(context: &Context) -> StructType {
        context.struct_type(
            &[
                context.i8_type().ptr_type(Default::default()).into(),
                context.i64_type().into(),
                context.i64_type().into(),
            ],
            false,
        )
    }

    /// Error abstraction: { has_error: i1, message_ptr: i8* }
    pub fn error_type(context: &Context) -> StructType {
        context.struct_type(
            &[
                context.bool_type().into(),
                context.i8_type().ptr_type(Default::default()).into(),
            ],
            false,
        )
    }

    /// Array wrapper: { data_ptr: i8*, len: i64, cap: i64 }
    pub fn array_type(context: &Context) -> StructType {
        context.struct_type(
            &[
                context.i8_type().ptr_type(Default::default()).into(),
                context.i64_type().into(),
                context.i64_type().into(),
            ],
            false,
        )
    }

    /// Convert mlang AST Type to LLVM BasicTypeEnum.
    pub fn mlang_type_to_llvm<'ctx>(
        mlang_type: &Type,
        context: &'ctx Context,
    ) -> Option<BasicTypeEnum<'ctx>> {
        match mlang_type {
            Type::Kain => Some(BasicTypeEnum::IntType(context.i64_type())),
            Type::Sar => Some(BasicTypeEnum::StructType(string_type(context))),
            Type::Sit => Some(BasicTypeEnum::IntType(context.bool_type())),
            Type::DaTha => Some(BasicTypeEnum::FloatType(context.f64_type())),
            Type::Nil => Some(BasicTypeEnum::IntType(context.i8_type())),
            Type::Error => Some(BasicTypeEnum::StructType(error_type(context))),
            Type::Array(_) => Some(BasicTypeEnum::StructType(array_type(context))),
            Type::Tuple(types) => {
                let elem_types: Vec<BasicTypeEnum> = types
                    .iter()
                    .filter_map(|t| mlang_type_to_llvm(t, context))
                    .collect();

                if elem_types.len() != types.len() {
                    return None;
                }

                Some(BasicTypeEnum::StructType(
                    context.struct_type(&elem_types, false),
                ))
            }
            Type::Struct(_)
            | Type::Function { .. }
            | Type::Interface(_)
            | Type::Map(_, _)
            | Type::Channel(_)
            | Type::Baung => None,
        }
    }

    pub fn mlang_type_to_llvm_ptr<'ctx>(
        mlang_type: &Type,
        context: &'ctx Context,
    ) -> Option<PointerType<'ctx>> {
        mlang_type_to_llvm(mlang_type, context).map(|basic| basic.ptr_type(Default::default()))
    }

    pub fn is_phase1_compatible(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Kain
                | Type::Sar
                | Type::Sit
                | Type::DaTha
                | Type::Nil
                | Type::Error
                | Type::Array(_)
                | Type::Tuple(_)
                | Type::Struct(_)
                | Type::Function { .. }
        )
    }
}

#[cfg(feature = "llvm-backend")]
pub use llvm_enabled::*;

#[cfg(not(feature = "llvm-backend"))]
pub fn llvm_backend_available() -> bool {
    false
}
