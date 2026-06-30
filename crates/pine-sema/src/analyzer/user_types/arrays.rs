use pine_builtins::{Accepts, BuiltinSignature};
use pine_ir::{PineType, Qualifier, ValueKind};
use pine_syntax::{CallArg, Diagnostic, Span};

use crate::analyzer::context::Analyzer;
use crate::types::accepts_type;

use super::{UserTypeArrayElementInference, classify_user_type_array_element_names};

impl Analyzer {
    pub(crate) fn array_from_user_type_element_inference(
        &self,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<UserTypeArrayElementInference> {
        let mut type_names = Vec::new();
        let mut saw_user_type = false;
        for (arg, arg_type) in args.iter().zip(arg_types.iter().copied()) {
            let Some(arg_type) = arg_type else {
                continue;
            };
            if arg_type.kind != ValueKind::UserType {
                continue;
            }
            saw_user_type = true;
            let Some(type_name) = self.user_type_name_of_expr(&arg.value) else {
                return Some(UserTypeArrayElementInference::UnknownUserTypeName);
            };
            type_names.push(type_name);
        }
        if !saw_user_type {
            return None;
        }

        classify_user_type_array_element_names(&self.user_types, &type_names)
            .or(Some(UserTypeArrayElementInference::UnknownUserTypeName))
    }

    pub(crate) fn analyze_user_type_array_new_call(
        &mut self,
        name: &str,
        type_name: &str,
        span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<PineType> {
        if args.len() > 2 {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!("`{name}` expects at most 2 argument(s), got {}", args.len()),
                args.get(2).map_or(span, |arg| arg.span),
            ));
        }

        match classify_user_type_array_element_names(&self.user_types, &[type_name.to_owned()]) {
            Some(UserTypeArrayElementInference::SameScalarLocal(_)) => {}
            Some(UserTypeArrayElementInference::UnsupportedFieldType(_)) => {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!("`{name}` does not support UDT arrays with non-scalar fields"),
                    span,
                ));
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!("`{name}` requires a local scalar-field UDT"),
                    span,
                ));
            }
        }

        for (index, arg) in args.iter().enumerate().take(2) {
            let expected_name = if index == 0 { "size" } else { "initial_value" };
            if let Some(name) = &arg.name
                && name != expected_name
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_NAME",
                    format!(
                        "`array.new<{type_name}>` has no argument named `{name}` at this position"
                    ),
                    arg.span,
                ));
                continue;
            }

            let Some(arg_type) = arg_types.get(index).copied().flatten() else {
                continue;
            };
            if index == 0 {
                if !accepts_type(Accepts::SimpleInt, arg_type) {
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_TYPE",
                        format!(
                            "`array.new<{type_name}>` argument `size` does not accept {:?} {:?}",
                            arg_type.qualifier, arg_type.kind
                        ),
                        arg.span,
                    ));
                }
            } else if arg_type.kind != ValueKind::UserType {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!(
                        "`array.new<{type_name}>` argument `initial_value` does not accept {:?} {:?}",
                        arg_type.qualifier, arg_type.kind
                    ),
                    arg.span,
                ));
            } else if let Some(value_type_name) = self.user_type_name_of_expr(&arg.value)
                && value_type_name != type_name
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!(
                        "`array.new<{type_name}>` argument `initial_value` expects UDT `{type_name}`, got `{value_type_name}`"
                    ),
                    arg.span,
                ));
            }
        }

        self.mark_expr_user_type_array(span, type_name.to_owned());
        Some(PineType::new(Qualifier::Simple, ValueKind::UserTypeArray))
    }

    pub(crate) fn mark_user_type_array_element_result(
        &mut self,
        signature_name: &str,
        span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        if !matches!(
            signature_name,
            "array.get"
                | "array.pop"
                | "array.remove"
                | "array.shift"
                | "array.first"
                | "array.last"
        ) {
            return;
        }
        let Some(array_type) = arg_types.first().copied().flatten() else {
            return;
        };
        if array_type.kind != ValueKind::UserTypeArray {
            return;
        }
        let Some(array_arg) = args.first() else {
            return;
        };
        if let Some(type_name) = self.user_type_array_name_of_expr(&array_arg.value) {
            self.mark_expr_user_type(span, type_name);
        }
    }

    pub(crate) fn mark_user_type_array_result(
        &mut self,
        signature_name: &str,
        span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        if !matches!(
            signature_name,
            "array.copy" | "array.concat" | "array.slice"
        ) {
            return;
        }
        let Some(array_type) = arg_types.first().copied().flatten() else {
            return;
        };
        if array_type.kind != ValueKind::UserTypeArray {
            return;
        }
        let Some(array_arg) = args.first() else {
            return;
        };
        if let Some(type_name) = self.user_type_array_name_of_expr(&array_arg.value) {
            self.mark_expr_user_type_array(span, type_name);
        }
    }

    pub(crate) fn validate_user_type_array_helper_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        if !signature.name.starts_with("array.")
            || matches!(
                signature.name,
                "array.size"
                    | "array.get"
                    | "array.set"
                    | "array.push"
                    | "array.insert"
                    | "array.pop"
                    | "array.remove"
                    | "array.shift"
                    | "array.unshift"
                    | "array.first"
                    | "array.last"
                    | "array.fill"
                    | "array.clear"
                    | "array.copy"
                    | "array.concat"
                    | "array.slice"
                    | "array.reverse"
                    | "array.join"
                    | "array.includes"
                    | "array.indexof"
                    | "array.lastindexof"
            )
        {
            return;
        }
        let Some((index, _)) = arg_types.iter().enumerate().find(|(_, arg_type)| {
            arg_type
                .map(|arg_type| arg_type.kind == ValueKind::UserTypeArray)
                .unwrap_or(false)
        }) else {
            return;
        };

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_TYPE",
            "`array.*` helper does not support UDT arrays except `array.size`, `array.get`, `array.set`, `array.push`, `array.insert`, `array.pop`, `array.remove`, `array.shift`, `array.unshift`, `array.first`, `array.last`, `array.fill`, `array.clear`, `array.copy`, `array.concat`, `array.slice`, `array.reverse`, `array.join`, `array.includes`, `array.indexof`, and `array.lastindexof`",
            args.get(index).map_or(Span::default(), |arg| arg.span),
        ));
    }

    pub(crate) fn validate_user_type_array_value_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        let value_index = match signature.name {
            "array.push" => 1,
            "array.unshift" => 1,
            "array.includes" => 1,
            "array.indexof" => 1,
            "array.lastindexof" => 1,
            "array.fill" => 1,
            "array.insert" => 2,
            "array.set" => 2,
            _ => return,
        };
        let Some(array_type) = arg_types.first().copied().flatten() else {
            return;
        };
        if array_type.kind != ValueKind::UserTypeArray {
            return;
        }

        let Some(value_arg) = args.get(value_index) else {
            return;
        };
        let Some(value_type) = arg_types.get(value_index).copied().flatten() else {
            return;
        };
        if value_type.kind != ValueKind::UserType {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{}` argument `value` does not accept {:?} {:?} for UDT arrays",
                    signature.name, value_type.qualifier, value_type.kind
                ),
                value_arg.span,
            ));
            return;
        }

        let Some(array_arg) = args.first() else {
            return;
        };
        let array_type_name = self.user_type_array_name_of_expr(&array_arg.value);
        let value_type_name = self.user_type_name_of_expr(&value_arg.value);
        if let (Some(array_type_name), Some(value_type_name)) = (array_type_name, value_type_name)
            && array_type_name != value_type_name
        {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{}` argument `value` expects UDT `{array_type_name}`, got `{value_type_name}`",
                    signature.name
                ),
                value_arg.span,
            ));
        }
    }

    pub(crate) fn validate_user_type_array_concat_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        if signature.name != "array.concat" {
            return;
        }
        let Some(first_type) = arg_types.first().copied().flatten() else {
            return;
        };
        let Some(second_type) = arg_types.get(1).copied().flatten() else {
            return;
        };
        if first_type.kind != ValueKind::UserTypeArray
            || second_type.kind != ValueKind::UserTypeArray
        {
            return;
        }
        let Some(first_arg) = args.first() else {
            return;
        };
        let Some(second_arg) = args.get(1) else {
            return;
        };
        let first_type_name = self.user_type_array_name_of_expr(&first_arg.value);
        let second_type_name = self.user_type_array_name_of_expr(&second_arg.value);
        if let (Some(first_type_name), Some(second_type_name)) = (first_type_name, second_type_name)
            && first_type_name != second_type_name
        {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`array.concat` argument `id2` expects UDT array `{first_type_name}`, got `{second_type_name}`"
                ),
                second_arg.span,
            ));
        }
    }
}
