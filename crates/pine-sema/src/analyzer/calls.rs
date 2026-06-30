use crate::analyzer::user_type_array_sort as ut_array_sort;
use crate::analyzer::user_types::UserTypeArrayElementInference;
use crate::prelude::*;

mod arrays;
mod declarations;
mod drawing_options;
mod helpers;
mod return_types;

pub(crate) use helpers::{
    array_method_builtin_name, drawing_method_builtin_name, expr_name, is_array_mutation_builtin,
    is_array_mutation_method_call_name, is_output_or_declaration_builtin,
    is_ta_extreme_length_overload, is_ta_pivot_default_source_overload, is_ta_vwap_bands_call,
    is_time_function_overload, is_timestamp_overload, method_call_parts, receiver_call_arg,
};

impl Analyzer {
    pub(crate) fn analyze_call(
        &mut self,
        callee: &Expr,
        args: &[CallArg],
        span: Span,
    ) -> Option<PineType> {
        let Some(name) = expr_name(callee) else {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_TARGET",
                "expected function name",
                callee.span,
            ));
            return None;
        };

        if name.starts_with("request.") {
            return self.analyze_request_call(&name, callee.span, args);
        }
        if let Some(constructor) = self.user_type_constructor(&name, args, span) {
            return Some(constructor.pine_type);
        }
        if let Some(constructor) = self.imported_user_type_constructor(&name, args, span) {
            return Some(constructor.pine_type);
        }
        let arg_types: Vec<_> = args
            .iter()
            .map(|arg| self.analyze_expr(&arg.value))
            .collect();

        if let Some(type_name) = ut_array_sort::array_new_user_type_name(&name) {
            return self.analyze_user_type_array_new_call(&name, type_name, span, args, &arg_types);
        }
        if ut_array_sort::is_user_type_array_ordering_call(&name, &arg_types) {
            return self.analyze_user_type_array_sort_call(&name, span, args, &arg_types);
        }

        if let Some(signature) = pine_builtins::get_phase_1_builtin(&name) {
            self.check_feature_name(&name, callee.span);
            self.validate_script_declaration_call(&name, callee.span, args);
            self.validate_strategy_order_call(&name, callee.span, args);
            self.validate_strategy_trade_field_call(&name, callee.span);
            if self.function_depth > 0 && is_output_or_declaration_builtin(&name) {
                self.unsupported(
                    "function_side_effect",
                    "indicator, strategy, input, plot, plotchar, plotshape, plotarrow, plotbar, plotcandle, hline, fill, bgcolor, barcolor, alert, alertcondition, drawing calls, and strategy order calls are not supported inside user-defined functions",
                    callee.span,
                );
            }
            if self.function_depth > 0 && is_array_mutation_builtin(&name) {
                self.unsupported(
                    "function_side_effect",
                    "collection mutation is not supported inside user-defined functions",
                    callee.span,
                );
            }

            self.validate_call_args(signature, args, &arg_types);
            if is_ta_vwap_bands_call(&name, args) {
                return Some(pine_builtins::tuple_return_type());
            }
            if name == "array.from"
                && let Some(UserTypeArrayElementInference::SameScalarLocal(type_name)) =
                    self.array_from_user_type_element_inference(args, &arg_types)
            {
                let pine_type = PineType::new(Qualifier::Simple, ValueKind::UserTypeArray);
                self.mark_expr_user_type_array(span, type_name);
                return Some(pine_type);
            }
            self.mark_user_type_array_element_result(&name, span, args, &arg_types);
            self.mark_user_type_array_result(&name, span, args, &arg_types);
            return self.return_type(signature, &arg_types);
        }

        match self.analyze_method_call(callee, span, args, &arg_types) {
            MethodResolution::Resolved(pine_type) => return pine_type,
            MethodResolution::NotMethod => {}
        }

        if self.functions.contains_key(&name) {
            return self.analyze_udf_call(&name, callee.span, span, args, &arg_types);
        }

        if let Some(reason) = unsupported_strategy_reason(&name) {
            self.unsupported(&name, reason, callee.span);
            return None;
        }
        if let Some(reason) = unsupported_log_reason(&name) {
            self.unsupported(&name, reason, callee.span);
            return None;
        }
        if let Some(reason) = unsupported_collection_reason(&name) {
            self.unsupported(&name, reason, callee.span);
            return None;
        }

        let unsupported_count = self.compatibility.unsupported.len();
        self.check_feature_name(&name, callee.span);
        if self.compatibility.unsupported.len() > unsupported_count {
            return None;
        }
        self.diagnostics.push(Diagnostic::error(
            "E_UNKNOWN_FUNCTION",
            format!("unknown function `{name}`"),
            callee.span,
        ));
        None
    }

    pub(crate) fn analyze_method_call(
        &mut self,
        callee: &Expr,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> MethodResolution {
        let Some((receiver_name, method_name)) = method_call_parts(callee) else {
            return MethodResolution::NotMethod;
        };
        if self.scope.resolve(receiver_name).is_none() {
            return MethodResolution::NotMethod;
        }

        let receiver_arg = receiver_call_arg(receiver_name, callee.span);
        let receiver_type = self.analyze_expr(&receiver_arg.value);
        let Some(receiver_type) = receiver_type else {
            return MethodResolution::Resolved(None);
        };
        if receiver_type.kind == ValueKind::UserType {
            return MethodResolution::Resolved(
                self.analyze_user_method_call(
                    receiver_name,
                    method_name,
                    callee.span,
                    call_span,
                    args,
                    arg_types,
                )
                .unwrap_or(None),
            );
        }
        if let Some(builtin_name) = drawing_method_builtin_name(receiver_type.kind, method_name) {
            let signature = pine_builtins::get_phase_1_builtin(&builtin_name)
                .expect("drawing method helper returned registered builtin");
            self.check_feature_name(&builtin_name, callee.span);

            if self.function_depth > 0 && is_output_or_declaration_builtin(&builtin_name) {
                self.unsupported(
                    "function_side_effect",
                    "indicator, strategy, input, plot, plotchar, plotshape, plotarrow, plotbar, plotcandle, hline, fill, bgcolor, barcolor, alert, alertcondition, drawing calls, and strategy order calls are not supported inside user-defined functions",
                    callee.span,
                );
            }

            let mut method_args = Vec::with_capacity(args.len() + 1);
            method_args.push(receiver_arg);
            method_args.extend(args.iter().cloned());
            let mut method_arg_types = Vec::with_capacity(arg_types.len() + 1);
            method_arg_types.push(Some(receiver_type));
            method_arg_types.extend(arg_types.iter().copied());

            self.validate_call_args(signature, &method_args, &method_arg_types);
            return MethodResolution::Resolved(self.return_type(signature, &method_arg_types));
        }

        if matches!(
            receiver_type.kind,
            ValueKind::Label
                | ValueKind::Line
                | ValueKind::LineFill
                | ValueKind::Box
                | ValueKind::Table
        ) {
            let namespace = match receiver_type.kind {
                ValueKind::Label => "label",
                ValueKind::Line => "line",
                ValueKind::LineFill => "linefill",
                ValueKind::Box => "box",
                ValueKind::Table => "table",
                _ => unreachable!("drawing receiver kind checked above"),
            };
            self.unsupported(
                &format!("{namespace}.{method_name}"),
                "this drawing object call is not supported in the current partial drawing subset",
                callee.span,
            );
            return MethodResolution::Resolved(None);
        }

        let builtin_name =
            matrix_method_builtin_name(receiver_type.kind, method_name).or_else(|| {
                is_array_kind(receiver_type.kind)
                    .then(|| array_method_builtin_name(method_name))
                    .flatten()
            });
        if builtin_name.is_none() && !is_array_kind(receiver_type.kind) {
            self.diagnostics.push(Diagnostic::error(
                "E_METHOD_RECEIVER_TYPE",
                format!(
                    "method `{method_name}` is not supported for {:?} {:?}",
                    receiver_type.qualifier, receiver_type.kind
                ),
                callee.span,
            ));
            return MethodResolution::Resolved(None);
        }

        let Some(signature) = builtin_name
            .and_then(|name| pine_builtins::get_phase_1_builtin(name).map(|sig| (name, sig)))
        else {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_METHOD",
                format!("unknown array method `{method_name}`"),
                callee.span,
            ));
            return MethodResolution::Resolved(None);
        };
        let (builtin_name, signature) = signature;
        self.check_feature_name(builtin_name, callee.span);

        if self.function_depth > 0 && is_array_mutation_builtin(builtin_name) {
            self.unsupported(
                "function_side_effect",
                "collection mutation is not supported inside user-defined functions",
                callee.span,
            );
        }

        let mut method_args = Vec::with_capacity(args.len() + 1);
        method_args.push(receiver_arg);
        method_args.extend(args.iter().cloned());
        let mut method_arg_types = Vec::with_capacity(arg_types.len() + 1);
        method_arg_types.push(Some(receiver_type));
        method_arg_types.extend(arg_types.iter().copied());
        if ut_array_sort::is_user_type_array_ordering_call(builtin_name, &method_arg_types) {
            return MethodResolution::Resolved(self.analyze_user_type_array_sort_call(
                builtin_name,
                call_span,
                &method_args,
                &method_arg_types,
            ));
        }

        self.validate_call_args(signature, &method_args, &method_arg_types);
        self.mark_user_type_array_element_result(
            builtin_name,
            call_span,
            &method_args,
            &method_arg_types,
        );
        self.mark_user_type_array_result(builtin_name, call_span, &method_args, &method_arg_types);
        MethodResolution::Resolved(self.return_type(signature, &method_arg_types))
    }

    pub(crate) fn validate_call_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        if is_time_function_overload(signature.name) {
            self.validate_time_function_args(signature, args, arg_types);
            return;
        }
        if is_timestamp_overload(signature.name) {
            self.validate_timestamp_args(signature, args, arg_types);
            return;
        }
        if self.validate_label_new_args(signature, args, arg_types) {
            return;
        }
        if self.validate_line_new_args(signature, args, arg_types) {
            return;
        }
        if self.validate_box_new_args(signature, args, arg_types) {
            return;
        }

        let required_count = signature
            .params
            .iter()
            .filter(|param| !param.optional)
            .count();
        let accepts_single_extreme_length =
            is_ta_extreme_length_overload(signature.name) && args.len() == 1;
        let accepts_pivot_default_source =
            is_ta_pivot_default_source_overload(signature.name) && args.len() == 2;
        if args.len() < required_count
            && !accepts_single_extreme_length
            && !accepts_pivot_default_source
        {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!(
                    "`{}` expects at least {} argument(s), got {}",
                    signature.name,
                    required_count,
                    args.len()
                ),
                args.first().map_or(Span::default(), |arg| arg.span),
            ));
            return;
        }

        if !signature.variadic && args.len() > signature.params.len() {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!(
                    "`{}` expects at most {} argument(s), got {}",
                    signature.name,
                    signature.params.len(),
                    args.len()
                ),
                args[signature.params.len()].span,
            ));
        }

        for (index, arg) in args.iter().enumerate() {
            if accepts_pivot_default_source {
                let expected_name = if index == 0 { "leftbars" } else { "rightbars" };
                if let Some(name) = &arg.name
                    && name != expected_name
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_NAME",
                        format!(
                            "`{}` two-argument overload has no argument named `{name}`",
                            signature.name
                        ),
                        arg.span,
                    ));
                    continue;
                }
                let Some(arg_type) = arg_types.get(index).copied().flatten() else {
                    continue;
                };
                if !accepts_type(Accepts::SimpleInt, arg_type) {
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_TYPE",
                        format!(
                            "`{}` argument `{expected_name}` does not accept {:?} {:?}",
                            signature.name, arg_type.qualifier, arg_type.kind
                        ),
                        arg.span,
                    ));
                }
                continue;
            }
            if accepts_single_extreme_length && index == 0 {
                if let Some(name) = &arg.name
                    && name != "length"
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_NAME",
                        format!(
                            "`{}` single-argument overload has no argument named `{name}`",
                            signature.name
                        ),
                        arg.span,
                    ));
                    continue;
                }
                let Some(arg_type) = arg_types.first().copied().flatten() else {
                    continue;
                };
                if !accepts_type(Accepts::SimpleInt, arg_type) {
                    self.diagnostics.push(Diagnostic::error(
                        "E_CALL_ARG_TYPE",
                        format!(
                            "`{}` argument `length` does not accept {:?} {:?}",
                            signature.name, arg_type.qualifier, arg_type.kind
                        ),
                        arg.span,
                    ));
                }
                continue;
            }
            let Some(param) = self.resolve_param(signature, index, arg) else {
                continue;
            };
            let Some(arg_type) = arg_types.get(index).copied().flatten() else {
                continue;
            };

            if signature.name == "array.join"
                && param.name == "id"
                && arg_type.kind == ValueKind::UserTypeArray
            {
                continue;
            }

            if !accepts_type(param.accepts, arg_type) {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!(
                        "`{}` argument `{}` does not accept {:?} {:?}",
                        signature.name, param.name, arg_type.qualifier, arg_type.kind
                    ),
                    arg.span,
                ));
            }
        }

        self.validate_user_type_array_value_args(signature, args, arg_types);
        self.validate_array_value_args(signature, args, arg_types);
        self.validate_array_concat_args(signature, args, arg_types);
        self.validate_user_type_array_concat_args(signature, args, arg_types);
        self.validate_array_from_args(signature, args, arg_types);
        self.validate_user_type_array_helper_args(signature, args, arg_types);
        self.validate_indicator_args(signature, args);
        self.validate_alert_args(signature, args);
        self.validate_drawing_option_args(signature, args);
    }

    pub(crate) fn resolve_param<'a>(
        &mut self,
        signature: &'a BuiltinSignature,
        index: usize,
        arg: &CallArg,
    ) -> Option<&'a pine_builtins::BuiltinParam> {
        if let Some(name) = &arg.name {
            let param = signature.params.iter().find(|param| param.name == name);
            if param.is_none() {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_NAME",
                    format!("`{}` has no argument named `{name}`", signature.name),
                    arg.span,
                ));
            }
            param
        } else {
            signature.params.get(index).or_else(|| {
                signature
                    .variadic
                    .then(|| signature.params.last())
                    .flatten()
            })
        }
    }
}
