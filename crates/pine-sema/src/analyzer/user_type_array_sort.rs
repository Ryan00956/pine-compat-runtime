use pine_builtins::Accepts;
use pine_ir::{PineType, Qualifier, ValueKind};
use pine_syntax::{CallArg, Diagnostic, Span};

use crate::analyzer::context::Analyzer;
use crate::types::{accepts_type, const_string_value};

pub(crate) fn array_new_user_type_name(name: &str) -> Option<&str> {
    let type_name = name.strip_prefix("array.new<")?.strip_suffix('>')?;
    (!type_name.contains('.')).then_some(type_name)
}

pub(crate) fn is_user_type_array_ordering_call(name: &str, arg_types: &[Option<PineType>]) -> bool {
    matches!(name, "array.sort" | "array.sort_indices")
        && matches!(
            arg_types.first().copied().flatten().map(|ty| ty.kind),
            Some(ValueKind::UserTypeArray)
        )
}

impl Analyzer {
    pub(crate) fn analyze_user_type_array_sort_call(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<PineType> {
        if args.len() < 3 {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!("`{name}` requires `sort_field` for UDT arrays"),
                span,
            ));
        }
        if args.len() > 3 {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!("`{name}` expects at most 3 argument(s), got {}", args.len()),
                args.get(3).map_or(span, |arg| arg.span),
            ));
        }
        for (index, arg) in args.iter().enumerate().take(3) {
            let expected_name = match index {
                0 => "id",
                1 => "order",
                _ => "sort_field",
            };
            if let Some(arg_name) = &arg.name
                && arg_name != expected_name
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_NAME",
                    format!("`{name}` has no argument named `{arg_name}` at this position"),
                    arg.span,
                ));
            }
        }
        if let Some(order_type) = arg_types.get(1).copied().flatten()
            && !accepts_type(Accepts::ConstString, order_type)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{name}` argument `order` does not accept {:?} {:?}",
                    order_type.qualifier, order_type.kind
                ),
                args.get(1).map_or(span, |arg| arg.span),
            ));
        }
        if let Some(field_type) = arg_types.get(2).copied().flatten()
            && !accepts_type(Accepts::ConstString, field_type)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{name}` argument `sort_field` does not accept {:?} {:?}",
                    field_type.qualifier, field_type.kind
                ),
                args.get(2).map_or(span, |arg| arg.span),
            ));
        }
        if self.user_type_array_sort_field_index(args).is_none() {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{name}` requires a local UDT array and an int, float, or string `sort_field`"
                ),
                args.get(2).map_or(span, |arg| arg.span),
            ));
        }
        let pine_type = if name == "array.sort_indices" {
            PineType::new(Qualifier::Simple, ValueKind::IntArray)
        } else {
            PineType::new(Qualifier::Const, ValueKind::Void)
        };
        Some(pine_type)
    }

    pub(crate) fn user_type_array_sort_field_index(&self, args: &[CallArg]) -> Option<usize> {
        let type_name = self.user_type_array_name_of_expr(&args.first()?.value)?;
        let field_name = const_string_value(&args.get(2)?.value)?;
        let (index, field) = self.user_type_field(&type_name, &field_name)?;
        matches!(
            field.pine_type.kind,
            ValueKind::Int | ValueKind::Float | ValueKind::String
        )
        .then_some(index)
    }
}
