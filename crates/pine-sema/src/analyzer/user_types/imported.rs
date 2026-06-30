use pine_ir::{PineType, Qualifier, ValueKind};
use pine_syntax::{CallArg, Diagnostic, Span};

use super::{
    ImportedUdtConstructorArgError, ImportedUdtConstructorArgPlan, UdtConstructor,
    UdtFieldAccessStep, UserTypeIdentity,
};
use crate::analyzer::context::Analyzer;
use crate::compatibility::FeatureUse;
use crate::types::{UNKNOWN, can_assign, strongest_qualifier};

impl Analyzer {
    pub(crate) fn imported_user_type_constructor_metadata(
        &self,
        callee_name: &str,
    ) -> Option<&crate::modules::ImportedUserTypeInfo> {
        let type_name = callee_name.strip_suffix(".new")?;
        self.imported_user_types.get(type_name)
    }

    pub(crate) fn imported_user_type_has_scalar_fields(
        &self,
        user_type: &crate::modules::ImportedUserTypeInfo,
    ) -> bool {
        user_type
            .fields
            .iter()
            .all(|field| field.pine_type.is_some())
    }

    pub(crate) fn imported_user_type_constructor_has_scalar_fields(
        &self,
        callee_name: &str,
    ) -> Option<bool> {
        let user_type = self.imported_user_type_constructor_metadata(callee_name)?;
        Some(self.imported_user_type_has_scalar_fields(user_type))
    }

    pub(crate) fn imported_user_type_constructor_arg_plan(
        &self,
        callee_name: &str,
        args: &[CallArg],
    ) -> Option<Result<ImportedUdtConstructorArgPlan, ImportedUdtConstructorArgError>> {
        let user_type = self.imported_user_type_constructor_metadata(callee_name)?;
        if args.len() > user_type.fields.len() {
            return Some(Err(ImportedUdtConstructorArgError::TooManyArgs {
                expected: user_type.fields.len(),
                actual: args.len(),
            }));
        }

        let mut resolved = vec![None; user_type.fields.len()];
        let mut positional_open = true;
        let mut next_positional = 0;
        for (arg_index, arg) in args.iter().enumerate() {
            if let Some(name) = &arg.name {
                positional_open = false;
                let Some(index) = user_type
                    .fields
                    .iter()
                    .position(|field| field.name == *name)
                else {
                    return Some(Err(ImportedUdtConstructorArgError::UnknownField(
                        name.clone(),
                    )));
                };
                if resolved[index].is_some() {
                    return Some(Err(ImportedUdtConstructorArgError::DuplicateField(
                        name.clone(),
                    )));
                }
                resolved[index] = Some(arg_index);
            } else {
                if !positional_open {
                    return Some(Err(ImportedUdtConstructorArgError::PositionalAfterNamed));
                }
                resolved[next_positional] = Some(arg_index);
                next_positional += 1;
            }
        }

        if let Some((index, _)) = resolved.iter().enumerate().find(|(_, seen)| seen.is_none()) {
            return Some(Err(ImportedUdtConstructorArgError::MissingField(
                user_type.fields[index].name.clone(),
            )));
        }
        Some(Ok(ImportedUdtConstructorArgPlan {
            scalar_fields: self
                .imported_user_type_constructor_has_scalar_fields(callee_name)
                .unwrap_or(false),
            field_arg_indices: resolved.into_iter().flatten().collect(),
        }))
    }

    pub(crate) fn imported_user_type_constructor(
        &mut self,
        callee_name: &str,
        args: &[CallArg],
        span: Span,
    ) -> Option<UdtConstructor> {
        let type_name = callee_name.strip_suffix(".new")?;
        let user_type = self.imported_user_types.get(type_name)?.clone();
        let plan = match self.imported_user_type_constructor_arg_plan(callee_name, args)? {
            Ok(plan) => plan,
            Err(error) => {
                self.diagnostics.push(imported_constructor_arg_diagnostic(
                    callee_name,
                    error,
                    span,
                ));
                return Some(imported_error_constructor(&user_type));
            }
        };
        if !plan.scalar_fields {
            self.diagnostics.push(Diagnostic::error(
                "E_IMPORT_UNSUPPORTED_UDT",
                format!(
                    "imported UDT `{type_name}` is not supported; non-scalar or deferred field metadata remains unsupported"
                ),
                span,
            ));
            return Some(imported_error_constructor(&user_type));
        }

        let mut qualifier = Qualifier::Const;
        let mut field_args = Vec::with_capacity(plan.field_arg_indices.len());
        for (field_index, arg_index) in plan.field_arg_indices.iter().copied().enumerate() {
            let field = &user_type.fields[field_index];
            let arg = &args[arg_index];
            let arg_type = self.analyze_expr(&arg.value).unwrap_or(UNKNOWN);
            let expected_type = field
                .pine_type
                .expect("scalar imported UDT field has Pine type metadata");
            if !can_assign(expected_type, arg_type) {
                self.diagnostics.push(Diagnostic::error(
                    "E_UDT_CONSTRUCTOR_ARG",
                    format!(
                        "cannot assign {:?} {:?} to imported field `{}` of type {:?}",
                        arg_type.qualifier, arg_type.kind, field.name, expected_type.kind
                    ),
                    arg.span,
                ));
            }
            qualifier = strongest_qualifier(qualifier, arg_type.qualifier);
            field_args.push(arg.value.clone());
        }

        let pine_type = PineType::new(qualifier, ValueKind::UserType);
        self.mark_expr_user_type(span, type_name.to_owned());
        self.compatibility.supported.push(FeatureUse {
            feature: "user-defined types".to_owned(),
            span,
        });
        Some(UdtConstructor {
            identity: UserTypeIdentity {
                source_id: user_type.identity.source_id,
                name: user_type.identity.name,
            },
            field_args,
            pine_type,
        })
    }

    pub(crate) fn imported_user_type_constructor_for_lowering(
        &self,
        callee_name: &str,
        args: &[CallArg],
        param_types: &std::collections::HashMap<String, PineType>,
    ) -> Option<UdtConstructor> {
        let type_name = callee_name.strip_suffix(".new")?;
        let user_type = self.imported_user_types.get(type_name)?;
        let plan = self
            .imported_user_type_constructor_arg_plan(callee_name, args)?
            .ok()?;
        if !plan.scalar_fields {
            return None;
        }
        Some(UdtConstructor {
            identity: UserTypeIdentity {
                source_id: user_type.identity.source_id,
                name: user_type.identity.name.clone(),
            },
            field_args: plan
                .field_arg_indices
                .into_iter()
                .map(|arg_index| args[arg_index].value.clone())
                .collect(),
            pine_type: self.type_of_imported_user_type_constructor_with_params(
                callee_name,
                args,
                param_types,
            )?,
        })
    }

    pub(crate) fn type_of_imported_user_type_constructor_with_params(
        &self,
        callee_name: &str,
        args: &[CallArg],
        param_types: &std::collections::HashMap<String, PineType>,
    ) -> Option<PineType> {
        let plan = self
            .imported_user_type_constructor_arg_plan(callee_name, args)?
            .ok()?;
        if !plan.scalar_fields {
            return None;
        }
        let mut qualifier = Qualifier::Const;
        for arg in args {
            let arg_type = self.type_of_expr_with_params(&arg.value, param_types)?;
            qualifier = strongest_qualifier(qualifier, arg_type.qualifier);
        }
        Some(PineType::new(qualifier, ValueKind::UserType))
    }

    pub(crate) fn imported_user_type_field_path(
        &self,
        type_name: &str,
        qualifier: Qualifier,
        field_names: &[String],
    ) -> Option<(PineType, Option<String>, Vec<UdtFieldAccessStep>)> {
        let user_type = self.imported_user_types.get(type_name)?;
        if field_names.len() != 1 {
            return None;
        }
        let (index, field) = user_type
            .fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == field_names[0])?;
        let pine_type = PineType::new(qualifier, field.pine_type?.kind);
        Some((
            pine_type,
            None,
            vec![UdtFieldAccessStep { index, pine_type }],
        ))
    }

    pub(crate) fn resolve_imported_user_type_field_path(
        &mut self,
        type_name: &str,
        qualifier: Qualifier,
        field_names: &[String],
        span: Span,
    ) -> Option<(PineType, Option<String>, Vec<UdtFieldAccessStep>)> {
        let user_type = self.imported_user_types.get(type_name)?;
        let field_name = field_names.first()?;
        if !user_type
            .fields
            .iter()
            .any(|field| field.name == *field_name)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_UDT_UNKNOWN_FIELD",
                format!("unknown field `{field_name}` on `{type_name}`"),
                span,
            ));
            return None;
        }
        if field_names.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                "E_UDT_UNKNOWN_FIELD",
                format!("field `{field_name}` on `{type_name}` is not a user-defined type"),
                span,
            ));
            return None;
        }
        self.imported_user_type_field_path(type_name, qualifier, field_names)
    }
}

fn imported_error_constructor(user_type: &crate::modules::ImportedUserTypeInfo) -> UdtConstructor {
    UdtConstructor {
        identity: UserTypeIdentity {
            source_id: user_type.identity.source_id,
            name: user_type.identity.name.clone(),
        },
        field_args: Vec::new(),
        pine_type: UNKNOWN,
    }
}

fn imported_constructor_arg_diagnostic(
    callee_name: &str,
    error: ImportedUdtConstructorArgError,
    span: Span,
) -> Diagnostic {
    match error {
        ImportedUdtConstructorArgError::TooManyArgs { expected, actual } => Diagnostic::error(
            "E_UDT_CONSTRUCTOR_ARG",
            format!("`{callee_name}` expects {expected} field argument(s), got {actual}"),
            span,
        ),
        ImportedUdtConstructorArgError::UnknownField(name) => Diagnostic::error(
            "E_UDT_CONSTRUCTOR_ARG",
            format!("unknown field `{name}` for `{callee_name}` constructor"),
            span,
        ),
        ImportedUdtConstructorArgError::DuplicateField(name) => Diagnostic::error(
            "E_UDT_CONSTRUCTOR_ARG",
            format!("duplicate field `{name}` for `{callee_name}` constructor"),
            span,
        ),
        ImportedUdtConstructorArgError::PositionalAfterNamed => Diagnostic::error(
            "E_UDT_CONSTRUCTOR_ARG",
            "positional field argument cannot follow named field argument",
            span,
        ),
        ImportedUdtConstructorArgError::MissingField(name) => Diagnostic::error(
            "E_UDT_CONSTRUCTOR_ARG",
            format!("missing field `{name}` for `{callee_name}` constructor"),
            span,
        ),
    }
}
