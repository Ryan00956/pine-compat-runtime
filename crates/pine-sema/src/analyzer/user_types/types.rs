use std::collections::HashMap;

use pine_ir::{PineType, ValueKind};
use pine_syntax::{Expr, Span};

use crate::resolver::SymbolInfo;
use crate::source_graph::SourceId;

#[derive(Debug, Clone)]
pub(crate) struct UserTypeInfo {
    pub(crate) identity: UserTypeIdentity,
    pub(crate) name: String,
    pub(crate) fields: Vec<UserTypeFieldInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserTypeIdentity {
    pub(crate) source_id: SourceId,
    pub(crate) name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct UserTypeFieldInfo {
    pub(crate) name: String,
    pub(crate) pine_type: PineType,
    pub(crate) user_type_name: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct UdtConstructor {
    pub(crate) identity: UserTypeIdentity,
    pub(crate) field_args: Vec<Expr>,
    pub(crate) pine_type: PineType,
}

#[derive(Debug, Clone)]
pub(crate) struct UdtFieldAccess {
    pub(crate) receiver: String,
    pub(crate) fields: Vec<UdtFieldAccessStep>,
}

#[derive(Debug, Clone)]
pub(crate) struct UdtFieldAccessStep {
    pub(crate) index: usize,
    pub(crate) pine_type: PineType,
}

pub(crate) struct UdtFieldMutation {
    pub(crate) pine_type: PineType,
    pub(crate) user_type_name: Option<String>,
    pub(crate) receiver_symbol: SymbolInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UserTypeArrayElementInference {
    SameScalarLocal(String),
    MixedLocal,
    UnsupportedFieldType(String),
    UnknownUserTypeName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImportedUdtConstructorArgError {
    TooManyArgs { expected: usize, actual: usize },
    UnknownField(String),
    DuplicateField(String),
    PositionalAfterNamed,
    MissingField(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportedUdtConstructorArgPlan {
    pub(crate) scalar_fields: bool,
    pub(crate) field_arg_indices: Vec<usize>,
}

pub(crate) fn span_key(span: Span) -> (usize, usize) {
    (span.start, span.end)
}

pub(crate) fn classify_user_type_array_element_names(
    user_types: &HashMap<String, UserTypeInfo>,
    type_names: &[String],
) -> Option<UserTypeArrayElementInference> {
    let first = type_names.first()?;
    if type_names.iter().any(|type_name| type_name != first) {
        return Some(UserTypeArrayElementInference::MixedLocal);
    }

    let user_type = user_types.get(first)?;
    debug_assert_eq!(user_type.identity.source_id, SourceId::root());
    debug_assert_eq!(user_type.identity.name, *first);
    if user_type.fields.iter().all(is_scalar_user_type_array_field) {
        Some(UserTypeArrayElementInference::SameScalarLocal(
            first.clone(),
        ))
    } else {
        Some(UserTypeArrayElementInference::UnsupportedFieldType(
            first.clone(),
        ))
    }
}

fn is_scalar_user_type_array_field(field: &UserTypeFieldInfo) -> bool {
    field.user_type_name.is_none()
        && matches!(
            field.pine_type.kind,
            ValueKind::Int
                | ValueKind::Float
                | ValueKind::Bool
                | ValueKind::String
                | ValueKind::Color
        )
}
