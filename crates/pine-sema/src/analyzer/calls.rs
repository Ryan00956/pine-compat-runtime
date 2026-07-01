use crate::analyzer::maps::{accepts_map_scalar_kind, map_kind_from_template_name};
use crate::analyzer::user_type_array_sort as ut_array_sort;
use crate::analyzer::user_types::UserTypeArrayElementInference;
use crate::prelude::*;
use crate::types::is_numeric_matrix_kind;

mod arrays;
mod declarations;
mod drawing_options;
mod helpers;
mod return_types;

pub(crate) use helpers::{
    alias_qualified_method_name, array_method_builtin_name, drawing_method_builtin_name, expr_name,
    is_array_mutation_builtin, is_array_mutation_method_call_name, is_map_mutation_builtin,
    is_map_mutation_method_call_name, is_output_or_declaration_builtin,
    is_ta_extreme_length_overload, is_ta_pivot_default_source_overload, is_ta_vwap_bands_call,
    is_time_function_overload, is_timestamp_overload, map_method_builtin_name, method_call_parts,
    receiver_call_arg,
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

        if let Some((key_type, value_type)) = map_new_template_types(&name) {
            return self.analyze_map_new_call(&name, key_type, value_type, span, args);
        }
        if let Some(pine_type) = self.analyze_map_operation_call(&name, span, args, &arg_types) {
            return pine_type;
        }
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
                    &unsupported_collection_mutation_udf_reason(&name),
                    callee.span,
                );
            }

            self.validate_call_args(signature, args, &arg_types);
            if is_ta_vwap_bands_call(&name, args) {
                return Some(pine_builtins::tuple_return_type());
            }
            if name == "array.from"
                && let Some(
                    UserTypeArrayElementInference::SameScalarLocal(type_name)
                    | UserTypeArrayElementInference::SameScalarImported(type_name),
                ) = self.array_from_user_type_element_inference(args, &arg_types)
            {
                let pine_type = PineType::new(Qualifier::Simple, ValueKind::UserTypeArray);
                self.mark_expr_user_type_array(span, type_name);
                return Some(pine_type);
            }
            self.mark_user_type_array_element_result(&name, span, args, &arg_types);
            self.mark_user_type_array_result(&name, span, args, &arg_types);
            return self.return_type(signature, &arg_types);
        }

        if let Some(pine_type) = self.analyze_alias_qualified_user_method_call(
            &name,
            callee.span,
            span,
            args,
            &arg_types,
        ) {
            return pine_type;
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

    fn analyze_map_new_call(
        &mut self,
        name: &str,
        key_type: &str,
        value_type: &str,
        span: Span,
        args: &[CallArg],
    ) -> Option<PineType> {
        if !args.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                "map.new does not accept arguments in the current subset",
                span,
            ));
            return None;
        }
        if !is_supported_map_scalar_type(key_type) || !is_supported_map_scalar_type(value_type) {
            self.unsupported(
                name,
                "map.new currently supports only int, float, bool, string, or color key/value templates",
                span,
            );
            return None;
        }
        self.compatibility.supported.push(FeatureUse {
            feature: "map.*".to_owned(),
            span,
        });
        self.mark_expr_map(
            span,
            MapTypeInfo {
                key_kind: map_kind_from_template_name(key_type)
                    .expect("supported map key type was checked"),
                value_kind: map_kind_from_template_name(value_type)
                    .expect("supported map value type was checked"),
            },
        );
        Some(PineType::new(Qualifier::Simple, ValueKind::Map))
    }

    fn analyze_map_operation_call(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> Option<Option<PineType>> {
        let expected_arity = match name {
            "map.put" => 3,
            "map.get" | "map.contains" | "map.remove" | "map.put_all" => 2,
            "map.clear" | "map.copy" | "map.size" | "map.keys" | "map.values" => 1,
            _ => return None,
        };
        if args.len() != expected_arity {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARITY",
                format!(
                    "`{name}` expects {expected_arity} argument(s), got {}",
                    args.len()
                ),
                args.get(expected_arity).map_or(span, |arg| arg.span),
            ));
            return Some(None);
        }

        if matches!(name, "map.put" | "map.clear" | "map.remove" | "map.put_all")
            && self.function_depth > 0
        {
            self.unsupported(
                "function_side_effect",
                &unsupported_collection_mutation_udf_reason(name),
                span,
            );
            return Some(None);
        }

        let Some(receiver_type) = arg_types.first().copied().flatten() else {
            return Some(None);
        };
        if receiver_type.kind != ValueKind::Map {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{name}` argument `id` does not accept {:?} {:?}",
                    receiver_type.qualifier, receiver_type.kind
                ),
                args[0].span,
            ));
            return Some(None);
        }
        let Some(map_info) = self.map_type_of_expr(&args[0].value) else {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!("`{name}` receiver map template is not known"),
                args[0].span,
            ));
            return Some(None);
        };

        if name == "map.put_all" {
            let Some(source_type) = arg_types.get(1).copied().flatten() else {
                return Some(None);
            };
            if source_type.kind != ValueKind::Map {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!(
                        "`{name}` argument `source` does not accept {:?} {:?}",
                        source_type.qualifier, source_type.kind
                    ),
                    args[1].span,
                ));
                return Some(None);
            }
            let Some(source_info) = self.map_type_of_expr(&args[1].value) else {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!("`{name}` source map template is not known"),
                    args[1].span,
                ));
                return Some(None);
            };
            if source_info != map_info {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    format!(
                        "`{name}` source map template {:?}/{:?} does not match target {:?}/{:?}",
                        source_info.key_kind,
                        source_info.value_kind,
                        map_info.key_kind,
                        map_info.value_kind
                    ),
                    args[1].span,
                ));
                return Some(None);
            }
        }

        if matches!(name, "map.put" | "map.get" | "map.contains" | "map.remove")
            && let Some(key_type) = arg_types.get(1).copied().flatten()
            && !accepts_map_scalar_kind(map_info.key_kind, key_type)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{name}` argument `key` does not accept {:?} {:?}",
                    key_type.qualifier, key_type.kind
                ),
                args[1].span,
            ));
        }

        if name == "map.put"
            && let Some(value_type) = arg_types.get(2).copied().flatten()
            && !accepts_map_scalar_kind(map_info.value_kind, value_type)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_CALL_ARG_TYPE",
                format!(
                    "`{name}` argument `value` does not accept {:?} {:?}",
                    value_type.qualifier, value_type.kind
                ),
                args[2].span,
            ));
        }

        self.compatibility.supported.push(FeatureUse {
            feature: "map.*".to_owned(),
            span,
        });

        if name == "map.copy" {
            self.mark_expr_map(span, map_info);
        }

        Some(Some(match name {
            "map.put" => PineType::new(Qualifier::Series, ValueKind::Void),
            "map.get" => PineType::new(Qualifier::Series, map_info.value_kind),
            "map.contains" => PineType::new(Qualifier::Series, ValueKind::Bool),
            "map.clear" => PineType::new(Qualifier::Series, ValueKind::Void),
            "map.remove" => PineType::new(Qualifier::Series, ValueKind::Void),
            "map.copy" => PineType::new(Qualifier::Simple, ValueKind::Map),
            "map.put_all" => PineType::new(Qualifier::Series, ValueKind::Void),
            "map.size" => PineType::new(Qualifier::Series, ValueKind::Int),
            "map.keys" => PineType::new(
                Qualifier::Simple,
                map_info
                    .key_kind
                    .array_kind_from_element_kind()
                    .expect("supported map key kind has array kind"),
            ),
            "map.values" => PineType::new(
                Qualifier::Simple,
                map_info
                    .value_kind
                    .array_kind_from_element_kind()
                    .expect("supported map value kind has array kind"),
            ),
            _ => unreachable!("map operation was matched above"),
        }))
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

        if receiver_type.kind == ValueKind::Map {
            let Some(builtin_name) = map_method_builtin_name(method_name) else {
                self.diagnostics.push(Diagnostic::error(
                    "E_UNKNOWN_METHOD",
                    format!("unknown map method `{method_name}`"),
                    callee.span,
                ));
                return MethodResolution::Resolved(None);
            };

            let mut method_args = Vec::with_capacity(args.len() + 1);
            method_args.push(receiver_arg);
            method_args.extend(args.iter().cloned());
            let mut method_arg_types = Vec::with_capacity(arg_types.len() + 1);
            method_arg_types.push(Some(receiver_type));
            method_arg_types.extend(arg_types.iter().copied());
            return MethodResolution::Resolved(
                self.analyze_map_operation_call(
                    builtin_name,
                    call_span,
                    &method_args,
                    &method_arg_types,
                )
                .unwrap_or(None),
            );
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
                &unsupported_collection_mutation_udf_reason(builtin_name),
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

            if !call_arg_accepts_type(signature, args, arg_types, param.accepts, arg_type) {
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
        self.validate_max_bars_back_args(signature, args);
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

fn map_new_template_types(name: &str) -> Option<(&str, &str)> {
    let inner = name.strip_prefix("map.new<")?.strip_suffix('>')?;
    inner.split_once(',')
}

fn is_supported_map_scalar_type(name: &str) -> bool {
    matches!(name, "int" | "float" | "bool" | "string" | "color")
}

fn call_arg_accepts_type(
    signature: &BuiltinSignature,
    args: &[CallArg],
    arg_types: &[Option<PineType>],
    accepts: Accepts,
    arg_type: PineType,
) -> bool {
    match accepts {
        Accepts::MatrixElementCompatible(matrix_param_index) => {
            let Some(matrix_type) =
                arg_type_for_param_index(signature, args, arg_types, matrix_param_index)
            else {
                return true;
            };
            accepts_matrix_element_arg(matrix_type, arg_type).unwrap_or(true)
        }
        Accepts::MatrixElementArray(matrix_param_index) => {
            let Some(matrix_type) =
                arg_type_for_param_index(signature, args, arg_types, matrix_param_index)
            else {
                return true;
            };
            accepts_matrix_element_array_arg(matrix_type, arg_type).unwrap_or(true)
        }
        Accepts::MatrixOrNumericCompatibleWithMatrixCounterpart(counterpart_param_index) => {
            if is_numeric_matrix_kind(arg_type.kind) {
                return true;
            }
            if !accepts_type(Accepts::NumericCompatible, arg_type) {
                return false;
            }
            let Some(counterpart_type) =
                arg_type_for_param_index(signature, args, arg_types, counterpart_param_index)
            else {
                return true;
            };
            is_numeric_matrix_kind(counterpart_type.kind)
        }
        Accepts::MatrixOrNumericOrNumericArrayCompatibleWithMatrixCounterpart(
            counterpart_param_index,
        ) => {
            if is_numeric_matrix_kind(arg_type.kind) {
                return true;
            }
            let accepts_compatible_value = accepts_type(Accepts::NumericCompatible, arg_type)
                || accepts_type(Accepts::NumericArray, arg_type);
            if !accepts_compatible_value {
                return false;
            }
            let Some(counterpart_type) =
                arg_type_for_param_index(signature, args, arg_types, counterpart_param_index)
            else {
                return true;
            };
            is_numeric_matrix_kind(counterpart_type.kind)
        }
        _ => accepts_type(accepts, arg_type),
    }
}

fn arg_type_for_param_index(
    signature: &BuiltinSignature,
    args: &[CallArg],
    arg_types: &[Option<PineType>],
    param_index: usize,
) -> Option<PineType> {
    args.iter().enumerate().find_map(|(arg_index, arg)| {
        (param_index_for_arg(signature, arg_index, arg)? == param_index)
            .then(|| arg_types.get(arg_index).copied().flatten())
            .flatten()
    })
}

fn param_index_for_arg(
    signature: &BuiltinSignature,
    arg_index: usize,
    arg: &CallArg,
) -> Option<usize> {
    if let Some(name) = &arg.name {
        return signature.params.iter().position(|param| param.name == name);
    }
    if arg_index < signature.params.len() {
        Some(arg_index)
    } else {
        signature.variadic.then_some(signature.params.len() - 1)
    }
}

fn unsupported_collection_mutation_udf_reason(operation: &str) -> String {
    format!("collection mutation via `{operation}` is not supported inside user-defined functions")
}
