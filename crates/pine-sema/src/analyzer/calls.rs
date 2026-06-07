use crate::prelude::*;

const LABEL_STYLES: &[&str] = &[
    "label.style_none",
    "label.style_xcross",
    "label.style_cross",
    "label.style_triangleup",
    "label.style_triangledown",
    "label.style_flag",
    "label.style_circle",
    "label.style_arrowup",
    "label.style_arrowdown",
    "label.style_label_up",
    "label.style_label_down",
    "label.style_label_left",
    "label.style_label_right",
    "label.style_label_lower_left",
    "label.style_label_lower_right",
    "label.style_label_upper_left",
    "label.style_label_upper_right",
];

const LABEL_SIZES: &[&str] = &[
    "size.auto",
    "size.tiny",
    "size.small",
    "size.normal",
    "size.large",
    "size.huge",
];

const LABEL_XLOCS: &[&str] = &["xloc.bar_index", "xloc.bar_time"];

const LABEL_YLOCS: &[&str] = &["yloc.price", "yloc.abovebar", "yloc.belowbar"];

const LINE_STYLES: &[&str] = &[
    "line.style_solid",
    "line.style_dotted",
    "line.style_dashed",
    "line.style_arrow_left",
    "line.style_arrow_right",
    "line.style_arrow_both",
];

const LINE_EXTENDS: &[&str] = &["extend.none", "extend.right", "extend.left", "extend.both"];

const TEXT_HALIGNS: &[&str] = &["text.align_left", "text.align_center", "text.align_right"];

const TEXT_VALIGNS: &[&str] = &["text.align_top", "text.align_center", "text.align_bottom"];

const TEXT_WRAPS: &[&str] = &["text.wrap_none", "text.wrap_auto"];

const TEXT_FONT_FAMILIES: &[&str] = &["font.family_default", "font.family_monospace"];

const TABLE_POSITIONS: &[&str] = &[
    "position.top_left",
    "position.top_center",
    "position.top_right",
    "position.middle_left",
    "position.middle_center",
    "position.middle_right",
    "position.bottom_left",
    "position.bottom_center",
    "position.bottom_right",
];

pub(crate) fn expr_name(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.clone()),
        ExprKind::QualifiedName(parts) => Some(parts.join(".")),
        _ => None,
    }
}
pub(crate) fn method_call_parts(expr: &Expr) -> Option<(&str, &str)> {
    match &expr.kind {
        ExprKind::QualifiedName(parts) if parts.len() == 2 => {
            Some((parts[0].as_str(), parts[1].as_str()))
        }
        _ => None,
    }
}
pub(crate) fn receiver_call_arg(receiver_name: &str, span: Span) -> CallArg {
    CallArg {
        name: None,
        span,
        value: Expr {
            kind: ExprKind::Identifier(receiver_name.to_owned()),
            span,
        },
    }
}
pub(crate) fn array_method_builtin_name(method_name: &str) -> Option<&'static str> {
    match method_name {
        "size" => Some("array.size"),
        "push" => Some("array.push"),
        "get" => Some("array.get"),
        "set" => Some("array.set"),
        "insert" => Some("array.insert"),
        "pop" => Some("array.pop"),
        "remove" => Some("array.remove"),
        "shift" => Some("array.shift"),
        "unshift" => Some("array.unshift"),
        "fill" => Some("array.fill"),
        "first" => Some("array.first"),
        "last" => Some("array.last"),
        "copy" => Some("array.copy"),
        "slice" => Some("array.slice"),
        "concat" => Some("array.concat"),
        "includes" => Some("array.includes"),
        "every" => Some("array.every"),
        "some" => Some("array.some"),
        "indexof" => Some("array.indexof"),
        "lastindexof" => Some("array.lastindexof"),
        "binary_search" => Some("array.binary_search"),
        "binary_search_leftmost" => Some("array.binary_search_leftmost"),
        "binary_search_rightmost" => Some("array.binary_search_rightmost"),
        "abs" => Some("array.abs"),
        "min" => Some("array.min"),
        "max" => Some("array.max"),
        "sum" => Some("array.sum"),
        "avg" => Some("array.avg"),
        "range" => Some("array.range"),
        "median" => Some("array.median"),
        "mode" => Some("array.mode"),
        "percentile_nearest_rank" => Some("array.percentile_nearest_rank"),
        "percentile_linear_interpolation" => Some("array.percentile_linear_interpolation"),
        "percentrank" => Some("array.percentrank"),
        "covariance" => Some("array.covariance"),
        "standardize" => Some("array.standardize"),
        "variance" => Some("array.variance"),
        "stdev" => Some("array.stdev"),
        "sort" => Some("array.sort"),
        "sort_indices" => Some("array.sort_indices"),
        "reverse" => Some("array.reverse"),
        "join" => Some("array.join"),
        "clear" => Some("array.clear"),
        _ => None,
    }
}
pub(crate) fn is_output_or_declaration_builtin(name: &str) -> bool {
    matches!(
        name,
        "indicator"
            | "strategy"
            | "alert"
            | "alertcondition"
            | "plot"
            | "hline"
            | "fill"
            | "bgcolor"
            | "barcolor"
            | "plotchar"
            | "plotshape"
            | "plotarrow"
            | "plotbar"
            | "plotcandle"
            | "label.new"
            | "label.set_x"
            | "label.set_xloc"
            | "label.set_y"
            | "label.set_xy"
            | "label.set_yloc"
            | "label.set_text"
            | "label.set_color"
            | "label.set_textcolor"
            | "label.set_style"
            | "label.set_size"
            | "label.set_tooltip"
            | "label.set_textalign"
            | "label.set_text_font_family"
            | "label.delete"
            | "label.copy"
            | "line.new"
            | "line.set_x1"
            | "line.set_y1"
            | "line.set_xy1"
            | "line.set_x2"
            | "line.set_y2"
            | "line.set_xy2"
            | "line.set_color"
            | "line.set_width"
            | "line.set_style"
            | "line.set_extend"
            | "line.delete"
            | "line.copy"
            | "box.new"
            | "box.set_left"
            | "box.set_top"
            | "box.set_right"
            | "box.set_bottom"
            | "box.set_lefttop"
            | "box.set_rightbottom"
            | "box.set_bgcolor"
            | "box.set_border_color"
            | "box.set_border_width"
            | "box.set_border_style"
            | "box.set_extend"
            | "box.set_text"
            | "box.set_text_color"
            | "box.set_text_size"
            | "box.set_text_halign"
            | "box.set_text_valign"
            | "box.set_text_wrap"
            | "box.set_text_font_family"
            | "box.delete"
            | "box.copy"
            | "table.new"
            | "table.delete"
            | "table.clear"
            | "table.cell"
            | "table.set_position"
            | "table.set_bgcolor"
            | "table.set_frame_color"
            | "table.set_frame_width"
            | "table.set_border_color"
            | "table.set_border_width"
            | "table.cell_set_text"
            | "table.cell_set_bgcolor"
            | "table.cell_set_text_color"
            | "table.cell_set_width"
            | "table.cell_set_height"
            | "table.cell_set_text_size"
            | "table.cell_set_text_halign"
            | "table.cell_set_text_valign"
            | "strategy.entry"
            | "strategy.close"
            | "strategy.close_all"
            | "strategy.cancel"
            | "strategy.cancel_all"
            | "strategy.exit"
    ) || name == "input"
        || name.starts_with("input.")
}
pub(crate) fn is_array_mutation_builtin(name: &str) -> bool {
    matches!(
        name,
        "array.push"
            | "array.set"
            | "array.insert"
            | "array.pop"
            | "array.remove"
            | "array.shift"
            | "array.unshift"
            | "array.fill"
            | "array.clear"
            | "array.sort"
            | "array.reverse"
            | "array.concat"
    )
}
pub(crate) fn is_array_mutation_method_call_name(name: &str) -> bool {
    name.rsplit_once('.')
        .and_then(|(_, method_name)| array_method_builtin_name(method_name))
        .is_some_and(is_array_mutation_builtin)
}
pub(crate) fn is_ta_extreme_length_overload(name: &str) -> bool {
    matches!(
        name,
        "ta.highest" | "ta.lowest" | "ta.highestbars" | "ta.lowestbars"
    )
}
pub(crate) fn is_ta_pivot_default_source_overload(name: &str) -> bool {
    matches!(name, "ta.pivothigh" | "ta.pivotlow")
}
pub(crate) fn is_ta_vwap_bands_call(name: &str, args: &[CallArg]) -> bool {
    name == "ta.vwap"
        && args.iter().enumerate().any(|(index, arg)| {
            arg.name.as_deref() == Some("stdev_mult") || (index >= 2 && arg.name.is_none())
        })
}

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

        let arg_types: Vec<_> = args
            .iter()
            .map(|arg| self.analyze_expr(&arg.value))
            .collect();

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
                    "array mutation is not supported inside user-defined functions",
                    callee.span,
                );
            }

            self.validate_call_args(signature, args, &arg_types);
            if is_ta_vwap_bands_call(&name, args) {
                return Some(pine_builtins::tuple_return_type());
            }
            return self.return_type(signature, &arg_types);
        }

        match self.analyze_method_call(callee, args, &arg_types) {
            MethodResolution::Resolved(pine_type) => return pine_type,
            MethodResolution::NotMethod => {}
        }

        if self.functions.contains_key(&name) {
            return self.analyze_udf_call(&name, callee.span, args, &arg_types);
        }

        if let Some(reason) = unsupported_strategy_reason(&name) {
            self.unsupported(&name, reason, callee.span);
            return None;
        }

        self.check_feature_name(&name, callee.span);
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
                    args,
                    arg_types,
                )
                .unwrap_or(None),
            );
        }
        if !is_array_kind(receiver_type.kind) {
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

        let builtin_name = array_method_builtin_name(method_name);
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
                "array mutation is not supported inside user-defined functions",
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
        MethodResolution::Resolved(self.return_type(signature, &method_arg_types))
    }

    pub(crate) fn validate_call_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
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

        self.validate_array_value_args(signature, args, arg_types);
        self.validate_array_concat_args(signature, args, arg_types);
        self.validate_array_from_args(signature, args, arg_types);
        self.validate_indicator_args(signature, args);
        self.validate_alert_args(signature, args);
        self.validate_label_new_args(signature, args);
    }

    pub(crate) fn validate_label_new_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
    ) {
        match signature.name {
            "label.new" => {
                self.validate_label_string_arg(signature, args, 3, "xloc", &["xloc.bar_index"]);
                self.validate_label_string_arg(signature, args, 4, "yloc", &["yloc.price"]);
                self.validate_label_string_arg(signature, args, 6, "style", LABEL_STYLES);
                self.validate_label_string_arg(signature, args, 8, "size", LABEL_SIZES);
            }
            "label.set_style" => {
                self.validate_label_string_arg(signature, args, 1, "style", LABEL_STYLES);
            }
            "label.set_size" => {
                self.validate_label_string_arg(signature, args, 1, "size", LABEL_SIZES);
            }
            "label.set_textalign" => {
                self.validate_label_string_arg(signature, args, 1, "textalign", TEXT_HALIGNS);
            }
            "label.set_text_font_family" => {
                self.validate_label_string_arg(
                    signature,
                    args,
                    1,
                    "text_font_family",
                    TEXT_FONT_FAMILIES,
                );
            }
            "label.set_xloc" => {
                self.validate_label_string_arg(signature, args, 2, "xloc", LABEL_XLOCS);
            }
            "label.set_yloc" => {
                self.validate_label_string_arg(signature, args, 1, "yloc", LABEL_YLOCS);
            }
            "line.set_style" => {
                self.validate_label_string_arg(signature, args, 1, "style", LINE_STYLES);
            }
            "line.set_extend" => {
                self.validate_label_string_arg(signature, args, 1, "extend", LINE_EXTENDS);
            }
            "box.set_extend" => {
                self.validate_label_string_arg(signature, args, 1, "extend", LINE_EXTENDS);
            }
            "box.set_border_style" => {
                self.validate_label_string_arg(signature, args, 1, "style", LINE_STYLES);
            }
            "box.set_text_halign" => {
                self.validate_label_string_arg(signature, args, 1, "text_halign", TEXT_HALIGNS);
            }
            "box.set_text_valign" => {
                self.validate_label_string_arg(signature, args, 1, "text_valign", TEXT_VALIGNS);
            }
            "box.set_text_wrap" => {
                self.validate_label_string_arg(signature, args, 1, "text_wrap", TEXT_WRAPS);
            }
            "box.set_text_font_family" => {
                self.validate_label_string_arg(
                    signature,
                    args,
                    1,
                    "text_font_family",
                    TEXT_FONT_FAMILIES,
                );
            }
            "table.new" => {
                self.validate_label_string_arg(signature, args, 0, "position", TABLE_POSITIONS);
            }
            "table.set_position" => {
                self.validate_label_string_arg(signature, args, 1, "position", TABLE_POSITIONS);
            }
            "table.cell_set_text_halign" => {
                self.validate_label_string_arg(signature, args, 3, "text_halign", TEXT_HALIGNS);
            }
            "table.cell_set_text_valign" => {
                self.validate_label_string_arg(signature, args, 3, "text_valign", TEXT_VALIGNS);
            }
            _ => {}
        }
    }

    pub(crate) fn validate_label_string_arg(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        index: usize,
        name: &str,
        allowed: &[&str],
    ) {
        for (arg_index, arg) in args.iter().enumerate() {
            let is_target = arg.name.as_deref() == Some(name)
                || (arg.name.is_none()
                    && signature
                        .params
                        .get(arg_index)
                        .is_some_and(|param| param.name == name && index == arg_index));
            if !is_target {
                continue;
            }
            let Some(value) = const_string_value(&arg.value) else {
                continue;
            };
            if !allowed.iter().any(|allowed_value| *allowed_value == value) {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_VALUE",
                    format!(
                        "`{}` argument `{name}` only supports {}",
                        signature.name,
                        allowed.join(", ")
                    ),
                    arg.span,
                ));
            }
        }
    }

    pub(crate) fn validate_indicator_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
    ) {
        if signature.name != "indicator" {
            return;
        }

        for (index, arg) in args.iter().enumerate() {
            let is_max_bars_back = arg.name.as_deref() == Some("max_bars_back")
                || (arg.name.is_none()
                    && signature
                        .params
                        .get(index)
                        .is_some_and(|param| param.name == "max_bars_back"));
            if !is_max_bars_back {
                continue;
            }

            if let Some(value) = const_int_value(&arg.value)
                && value < 0
            {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_VALUE",
                    "`indicator` argument `max_bars_back` must be non-negative",
                    arg.span,
                ));
            }
        }
    }

    pub(crate) fn validate_array_value_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        let value_index = match signature.name {
            "array.push"
            | "array.unshift"
            | "array.fill"
            | "array.includes"
            | "array.indexof"
            | "array.lastindexof"
            | "array.binary_search"
            | "array.binary_search_leftmost"
            | "array.binary_search_rightmost" => 1,
            "array.set" | "array.insert" => 2,
            _ => return,
        };
        let Some(array_type) = arg_types.first().copied().flatten() else {
            return;
        };
        let Some(value_type) = arg_types.get(value_index).copied().flatten() else {
            return;
        };
        let expected = match array_type.kind {
            ValueKind::FloatArray
                if matches!(
                    value_type.kind,
                    ValueKind::Int | ValueKind::Float | ValueKind::Na
                ) =>
            {
                return;
            }
            ValueKind::IntArray if matches!(value_type.kind, ValueKind::Int | ValueKind::Na) => {
                return;
            }
            ValueKind::BoolArray if matches!(value_type.kind, ValueKind::Bool | ValueKind::Na) => {
                return;
            }
            ValueKind::StringArray
                if matches!(value_type.kind, ValueKind::String | ValueKind::Na) =>
            {
                return;
            }
            ValueKind::ColorArray
                if matches!(value_type.kind, ValueKind::Color | ValueKind::Na) =>
            {
                return;
            }
            ValueKind::FloatArray => "float arrays",
            ValueKind::IntArray => "int arrays",
            ValueKind::BoolArray => "bool arrays",
            ValueKind::StringArray => "string arrays",
            ValueKind::ColorArray => "color arrays",
            _ => return,
        };

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_TYPE",
            format!(
                "`{}` argument `value` does not accept {:?} {:?} for {expected}",
                signature.name, value_type.qualifier, value_type.kind,
            ),
            args.get(value_index)
                .map_or(Span::default(), |arg| arg.span),
        ));
    }

    pub(crate) fn validate_array_concat_args(
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
        if !is_array_kind(first_type.kind)
            || !is_array_kind(second_type.kind)
            || first_type.kind == second_type.kind
        {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_TYPE",
            format!(
                "`array.concat` argument `id2` does not accept {:?} {:?} for {:?} {:?}",
                second_type.qualifier, second_type.kind, first_type.qualifier, first_type.kind,
            ),
            args.get(1).map_or(Span::default(), |arg| arg.span),
        ));
    }

    pub(crate) fn validate_array_from_args(
        &mut self,
        signature: &BuiltinSignature,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) {
        if signature.name != "array.from" {
            return;
        }
        if array_from_return_type(arg_types).is_some() {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_TYPE",
            "`array.from` arguments must infer one supported array element kind",
            args.first().map_or(Span::default(), |arg| arg.span),
        ));
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

    pub(crate) fn return_type(
        &self,
        signature: &BuiltinSignature,
        arg_types: &[Option<PineType>],
    ) -> Option<PineType> {
        match signature.returns {
            ReturnSpec::Fixed(pine_type) => Some(pine_type),
            ReturnSpec::Tuple(_) => Some(pine_builtins::tuple_return_type()),
            ReturnSpec::SameAsArg(index) => arg_types.get(index).copied().flatten(),
            ReturnSpec::BoolFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(pine_builtins::fallback_bool_for_arg),
            ReturnSpec::ColorFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(pine_builtins::color_return_for_arg),
            ReturnSpec::PromotedColor => promoted_color_type(arg_types),
            ReturnSpec::PromotedBool => promoted_bool_type(arg_types),
            ReturnSpec::PromotedInt => promoted_int_type(arg_types),
            ReturnSpec::PromotedString => promoted_string_type(arg_types),
            ReturnSpec::FloatFromStringArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(float_return_for_arg),
            ReturnSpec::PromotedNumeric => promoted_numeric_type(arg_types),
            ReturnSpec::ArrayElement(index) => array_element_return_type(arg_types, index),
            ReturnSpec::ArrayNumeric(index) => array_numeric_return_type(arg_types, index),
            ReturnSpec::ArrayFromArgs => array_from_return_type(arg_types),
            ReturnSpec::IntFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(int_return_for_arg),
            ReturnSpec::FloatFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .map(float_return_for_arg),
            ReturnSpec::SeriesFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .and_then(series_return_for_arg),
            ReturnSpec::ChangeFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .and_then(pine_builtins::change_return_for_arg),
            ReturnSpec::PromotedFloat => promoted_float_type(arg_types),
            ReturnSpec::Round => round_return_type(arg_types),
            ReturnSpec::InputFromArg(index) => arg_types
                .get(index)
                .copied()
                .flatten()
                .and_then(pine_builtins::input_return_for_arg),
        }
    }
}
