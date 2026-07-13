use crate::prelude::*;

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

pub(crate) fn postfix_call_result_method_parts<'a>(
    callee: &'a Expr,
    args: &[CallArg],
) -> Option<(&'a str, &'a str)> {
    let parts = method_call_parts(callee)?;
    let receiver = args.first()?;
    if receiver.name.is_some()
        || !matches!(receiver.value.kind, ExprKind::Call { .. })
        || receiver.value.span.end >= callee.span.start
    {
        return None;
    }
    Some(parts)
}

pub(crate) fn local_udf_call_result_method_parts<'a>(
    callee: &'a Expr,
    args: &'a [CallArg],
) -> Option<(&'a str, &'a str)> {
    let (prefix, method_name) = postfix_call_result_method_parts(callee, args)?;
    if prefix != "$call_result" {
        return None;
    }
    let ExprKind::Call {
        callee: producer,
        args: producer_args,
    } = &args.first()?.value.kind
    else {
        return None;
    };
    match &producer.kind {
        ExprKind::Identifier(function_name) => Some((function_name, method_name)),
        ExprKind::QualifiedName(_) => {
            let (function_name, producer_method) =
                local_udf_call_result_method_parts(producer, producer_args)?;
            matches!(
                producer_method,
                "copy"
                    | "diff"
                    | "eigenvectors"
                    | "inv"
                    | "kron"
                    | "mult"
                    | "pinv"
                    | "pow"
                    | "submatrix"
                    | "transpose"
            )
            .then_some((function_name, method_name))
        }
        _ => None,
    }
}

const BUILTIN_MATRIX_CALL_RESULT_PREFIX: &str = "$builtin_matrix_result";
const BUILTIN_MAP_CALL_RESULT_PREFIX: &str = "$builtin_map_result";

pub(crate) fn builtin_matrix_call_result_method_name<'a>(
    callee: &'a Expr,
    args: &[CallArg],
) -> Option<&'a str> {
    let (prefix, method_name) = postfix_call_result_method_parts(callee, args)?;
    (prefix == BUILTIN_MATRIX_CALL_RESULT_PREFIX).then_some(method_name)
}

pub(crate) fn builtin_map_call_result_method_name<'a>(
    callee: &'a Expr,
    args: &[CallArg],
) -> Option<&'a str> {
    let (prefix, method_name) = postfix_call_result_method_parts(callee, args)?;
    (prefix == BUILTIN_MAP_CALL_RESULT_PREFIX).then_some(method_name)
}

pub(crate) fn bound_matrix_call_result_method_parts<'a>(
    callee: &'a Expr,
    args: &'a [CallArg],
) -> Option<(&'a str, &'a str)> {
    let (prefix, method_name) = postfix_call_result_method_parts(callee, args)?;
    let ExprKind::Call {
        callee: producer, ..
    } = &args.first()?.value.kind
    else {
        return None;
    };
    let ExprKind::QualifiedName(parts) = &producer.kind else {
        return None;
    };
    match parts.as_slice() {
        [receiver_name, producer_method]
            if receiver_name == prefix
                && matches!(
                    producer_method.as_str(),
                    "copy"
                        | "diff"
                        | "eigenvectors"
                        | "inv"
                        | "kron"
                        | "mult"
                        | "pinv"
                        | "pow"
                        | "submatrix"
                        | "transpose"
                ) =>
        {
            Some((receiver_name, method_name))
        }
        _ => None,
    }
}

pub(crate) fn array_call_result_builtin_name(method_name: &str) -> Option<&'static str> {
    match method_name {
        "size" => Some("array.size"),
        "get" => Some("array.get"),
        "first" => Some("array.first"),
        "last" => Some("array.last"),
        "copy" => Some("array.copy"),
        "includes" => Some("array.includes"),
        "indexof" => Some("array.indexof"),
        "lastindexof" => Some("array.lastindexof"),
        "binary_search" => Some("array.binary_search"),
        "binary_search_leftmost" => Some("array.binary_search_leftmost"),
        "binary_search_rightmost" => Some("array.binary_search_rightmost"),
        _ => None,
    }
}

pub(crate) fn matrix_call_result_builtin_name(method_name: &str) -> Option<&'static str> {
    match method_name {
        "rows" => Some("matrix.rows"),
        "columns" => Some("matrix.columns"),
        "elements_count" => Some("matrix.elements_count"),
        "get" => Some("matrix.get"),
        "copy" => Some("matrix.copy"),
        "diff" => Some("matrix.diff"),
        "eigenvectors" => Some("matrix.eigenvectors"),
        "inv" => Some("matrix.inv"),
        "kron" => Some("matrix.kron"),
        "mult" => Some("matrix.mult"),
        "pinv" => Some("matrix.pinv"),
        "pow" => Some("matrix.pow"),
        "submatrix" => Some("matrix.submatrix"),
        "transpose" => Some("matrix.transpose"),
        "row" => Some("matrix.row"),
        "col" => Some("matrix.col"),
        "eigenvalues" => Some("matrix.eigenvalues"),
        "is_square" => Some("matrix.is_square"),
        "is_zero" => Some("matrix.is_zero"),
        "is_binary" => Some("matrix.is_binary"),
        "is_diagonal" => Some("matrix.is_diagonal"),
        "is_identity" => Some("matrix.is_identity"),
        "is_symmetric" => Some("matrix.is_symmetric"),
        "is_antisymmetric" => Some("matrix.is_antisymmetric"),
        "is_stochastic" => Some("matrix.is_stochastic"),
        "sum" => Some("matrix.sum"),
        "avg" => Some("matrix.avg"),
        "min" => Some("matrix.min"),
        "max" => Some("matrix.max"),
        "mode" => Some("matrix.mode"),
        "trace" => Some("matrix.trace"),
        "det" => Some("matrix.det"),
        "rank" => Some("matrix.rank"),
        _ => None,
    }
}

pub(crate) fn map_call_result_builtin_name(method_name: &str) -> Option<&'static str> {
    match method_name {
        "size" => Some("map.size"),
        "get" => Some("map.get"),
        "contains" => Some("map.contains"),
        "copy" => Some("map.copy"),
        "keys" => Some("map.keys"),
        "values" => Some("map.values"),
        _ => None,
    }
}

pub(crate) fn alias_qualified_method_name(name: &str) -> Option<(&str, &str)> {
    let (alias, method_name) = name.split_once('.')?;
    if alias.is_empty() || method_name.is_empty() || method_name.contains('.') {
        return None;
    }
    Some((alias, method_name))
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

pub(crate) fn call_arg_accepts_type_expected_diagnostic(
    function_name: &str,
    param_name: &str,
    accepts: Accepts,
    arg_type: PineType,
    span: Span,
) -> Option<Diagnostic> {
    if accepts_type(accepts, arg_type) {
        return None;
    }
    if let Some(expected) = accepts_expected_label(accepts) {
        return Some(call_arg_expected_type_diagnostic(
            function_name,
            param_name,
            &expected,
            arg_type,
            span,
        ));
    }
    Some(call_arg_type_diagnostic(
        function_name,
        param_name,
        arg_type,
        span,
    ))
}

fn accepts_expected_label(accepts: Accepts) -> Option<String> {
    match accepts {
        Accepts::Exact(expected) => Some(pine_type_name(expected)),
        Accepts::Kind(kind) => Some(value_kind_name(kind).to_owned()),
        Accepts::SeriesFloat => Some("series float".to_owned()),
        Accepts::SeriesNumeric => Some("series numeric".to_owned()),
        Accepts::SeriesNumericOrBool => Some("series numeric or bool".to_owned()),
        Accepts::SeriesOrSimpleNumeric => Some("series/simple numeric".to_owned()),
        Accepts::SeriesOrSimpleNumericOrBool => Some("series/simple numeric or bool".to_owned()),
        Accepts::Numeric => Some("numeric".to_owned()),
        Accepts::NumericCompatible => Some("numeric-compatible".to_owned()),
        Accepts::IntCompatible => Some("integer-compatible".to_owned()),
        Accepts::BoolCompatible => Some("bool-compatible".to_owned()),
        Accepts::StringCompatible => Some("string-compatible".to_owned()),
        Accepts::StringConvertible => Some("string-convertible".to_owned()),
        Accepts::CastScalar => Some("int/float/bool-compatible".to_owned()),
        Accepts::StringCastScalar => Some("int/float/bool/string-compatible".to_owned()),
        Accepts::ColorCompatible => Some("color-compatible".to_owned()),
        Accepts::NumericOrColorCompatible => Some("numeric/color-compatible".to_owned()),
        Accepts::LabelCompatible => Some("label-compatible".to_owned()),
        Accepts::LineCompatible => Some("line-compatible".to_owned()),
        Accepts::LineFillCompatible => Some("linefill-compatible".to_owned()),
        Accepts::PolylineCompatible => Some("polyline-compatible".to_owned()),
        Accepts::BoxCompatible => Some("box-compatible".to_owned()),
        Accepts::TableCompatible => Some("table-compatible".to_owned()),
        Accepts::ValueWhenSource => Some("numeric/bool/color-compatible".to_owned()),
        Accepts::StringOrIntCompatible => Some("string/int-compatible".to_owned()),
        Accepts::ChartPointCompatible => Some("chart.point-compatible".to_owned()),
        Accepts::PlotOrHLine => Some("plot/hline".to_owned()),
        Accepts::Map => Some("map".to_owned()),
        Accepts::Array => Some("array".to_owned()),
        Accepts::NumericArray => Some("numeric array".to_owned()),
        Accepts::NumericOrBoolArray => Some("numeric/bool array".to_owned()),
        Accepts::NumericOrStringArray => Some("numeric/string array".to_owned()),
        Accepts::ScalarArray => Some("scalar array".to_owned()),
        Accepts::Matrix => Some("matrix".to_owned()),
        Accepts::NumericMatrix => Some("numeric matrix".to_owned()),
        Accepts::FloatMatrix => Some("matrix<float>".to_owned()),
        Accepts::QualifierBoundScalar(bound) => Some(bound.expected_label()),
        Accepts::Tuple => Some("tuple".to_owned()),
        Accepts::InputDefval => Some("const int/float/bool/string/color".to_owned()),
        _ => None,
    }
}

pub(crate) fn call_arg_type_diagnostic(
    function_name: &str,
    param_name: &str,
    arg_type: PineType,
    span: Span,
) -> Diagnostic {
    Diagnostic::error(
        "E_CALL_ARG_TYPE",
        format!(
            "`{function_name}` argument `{param_name}` does not accept {}",
            pine_type_name(arg_type)
        ),
        span,
    )
}

pub(crate) fn call_arg_expected_type_diagnostic(
    function_name: &str,
    param_name: &str,
    expected: &str,
    arg_type: PineType,
    span: Span,
) -> Diagnostic {
    Diagnostic::error(
        "E_CALL_ARG_TYPE",
        format!(
            "`{function_name}` argument `{param_name}` expects {expected}, got {}",
            pine_type_name(arg_type)
        ),
        span,
    )
}

pub(crate) fn call_arg_expected_label_diagnostic(
    function_name: &str,
    param_name: &str,
    expected: &str,
    actual: &str,
    span: Span,
) -> Diagnostic {
    Diagnostic::error(
        "E_CALL_ARG_TYPE",
        format!("`{function_name}` argument `{param_name}` expects {expected}, got {actual}"),
        span,
    )
}

pub(crate) fn call_requirement_diagnostic(
    function_name: &str,
    requirement: &str,
    span: Span,
) -> Diagnostic {
    Diagnostic::error(
        "E_CALL_ARG_TYPE",
        format!("`{function_name}` requires {requirement}"),
        span,
    )
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

pub(crate) fn map_method_builtin_name(method_name: &str) -> Option<&'static str> {
    match method_name {
        "size" => Some("map.size"),
        "put" => Some("map.put"),
        "get" => Some("map.get"),
        "contains" => Some("map.contains"),
        "clear" => Some("map.clear"),
        "remove" => Some("map.remove"),
        "copy" => Some("map.copy"),
        "put_all" => Some("map.put_all"),
        "keys" => Some("map.keys"),
        "values" => Some("map.values"),
        _ => None,
    }
}

pub(crate) fn drawing_method_builtin_name(
    receiver_kind: ValueKind,
    method_name: &str,
) -> Option<String> {
    let namespace = match receiver_kind {
        ValueKind::Label => "label",
        ValueKind::Line => "line",
        ValueKind::LineFill => "linefill",
        ValueKind::Polyline => "polyline",
        ValueKind::Box => "box",
        ValueKind::Table => "table",
        _ => return None,
    };
    let builtin_name = format!("{namespace}.{method_name}");
    let signature = pine_builtins::get_phase_1_builtin(&builtin_name)?;
    let first_param = signature.params.first()?;
    let accepts_receiver = match receiver_kind {
        ValueKind::Label => first_param.accepts == Accepts::LabelCompatible,
        ValueKind::Line => first_param.accepts == Accepts::LineCompatible,
        ValueKind::LineFill => first_param.accepts == Accepts::LineFillCompatible,
        ValueKind::Polyline => first_param.accepts == Accepts::PolylineCompatible,
        ValueKind::Box => first_param.accepts == Accepts::BoxCompatible,
        ValueKind::Table => first_param.accepts == Accepts::TableCompatible,
        _ => false,
    };
    accepts_receiver.then_some(builtin_name)
}

pub(crate) fn is_output_or_declaration_builtin(name: &str) -> bool {
    matches!(
        name,
        "indicator"
            | "max_bars_back"
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
            | "label.set_point"
            | "label.set_yloc"
            | "label.set_text"
            | "label.set_color"
            | "label.set_textcolor"
            | "label.set_style"
            | "label.set_size"
            | "label.set_tooltip"
            | "label.set_textalign"
            | "label.set_text_font_family"
            | "label.set_text_formatting"
            | "label.delete"
            | "label.copy"
            | "line.new"
            | "line.set_first_point"
            | "line.set_x1"
            | "line.set_y1"
            | "line.set_xy1"
            | "line.set_second_point"
            | "line.set_x2"
            | "line.set_y2"
            | "line.set_xy2"
            | "line.set_xloc"
            | "line.set_color"
            | "line.set_width"
            | "line.set_style"
            | "line.set_extend"
            | "line.delete"
            | "line.copy"
            | "polyline.new"
            | "polyline.delete"
            | "box.new"
            | "box.set_left"
            | "box.set_top"
            | "box.set_right"
            | "box.set_bottom"
            | "box.set_lefttop"
            | "box.set_top_left_point"
            | "box.set_rightbottom"
            | "box.set_bottom_right_point"
            | "box.set_bgcolor"
            | "box.set_border_color"
            | "box.set_border_width"
            | "box.set_border_style"
            | "box.set_extend"
            | "box.set_xloc"
            | "box.set_text"
            | "box.set_text_color"
            | "box.set_text_size"
            | "box.set_text_halign"
            | "box.set_text_valign"
            | "box.set_text_wrap"
            | "box.set_text_font_family"
            | "box.set_text_formatting"
            | "box.delete"
            | "box.copy"
            | "table.new"
            | "table.delete"
            | "table.clear"
            | "table.merge_cells"
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
            | "table.cell_set_tooltip"
            | "table.cell_set_text_font_family"
            | "table.cell_set_text_formatting"
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
        "matrix.set"
            | "matrix.fill"
            | "matrix.reshape"
            | "matrix.reverse"
            | "matrix.add_row"
            | "matrix.add_col"
            | "matrix.remove_col"
            | "matrix.remove_row"
            | "matrix.sort"
            | "matrix.swap_columns"
            | "matrix.swap_rows"
    ) || matches!(
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

pub(crate) fn is_map_mutation_builtin(name: &str) -> bool {
    matches!(name, "map.put" | "map.clear" | "map.remove" | "map.put_all")
}

pub(crate) fn is_map_mutation_method_call_name(name: &str) -> bool {
    name.rsplit_once('.')
        .and_then(|(_, method_name)| map_method_builtin_name(method_name))
        .is_some_and(is_map_mutation_builtin)
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

pub(crate) fn is_time_function_overload(name: &str) -> bool {
    matches!(name, "time" | "time_close")
}

pub(crate) fn is_timestamp_overload(name: &str) -> bool {
    name == "timestamp"
}

pub(crate) fn is_ta_vwap_bands_call(name: &str, args: &[CallArg]) -> bool {
    name == "ta.vwap"
        && args.iter().enumerate().any(|(index, arg)| {
            arg.name.as_deref() == Some("stdev_mult") || (index >= 2 && arg.name.is_none())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_builtins::{QualifierBoundScalar, ScalarKind};

    fn pine_type(qualifier: Qualifier, kind: ValueKind) -> PineType {
        PineType::new(qualifier, kind)
    }

    #[test]
    fn matrix_call_result_helpers_are_a_closed_registered_set() {
        for (method_name, builtin_name) in [
            ("rows", "matrix.rows"),
            ("columns", "matrix.columns"),
            ("elements_count", "matrix.elements_count"),
            ("get", "matrix.get"),
            ("copy", "matrix.copy"),
            ("diff", "matrix.diff"),
            ("eigenvectors", "matrix.eigenvectors"),
            ("inv", "matrix.inv"),
            ("kron", "matrix.kron"),
            ("mult", "matrix.mult"),
            ("pinv", "matrix.pinv"),
            ("pow", "matrix.pow"),
            ("submatrix", "matrix.submatrix"),
            ("transpose", "matrix.transpose"),
            ("row", "matrix.row"),
            ("col", "matrix.col"),
            ("eigenvalues", "matrix.eigenvalues"),
            ("is_square", "matrix.is_square"),
            ("is_zero", "matrix.is_zero"),
            ("is_binary", "matrix.is_binary"),
            ("is_diagonal", "matrix.is_diagonal"),
            ("is_identity", "matrix.is_identity"),
            ("is_symmetric", "matrix.is_symmetric"),
            ("is_antisymmetric", "matrix.is_antisymmetric"),
            ("is_stochastic", "matrix.is_stochastic"),
            ("sum", "matrix.sum"),
            ("avg", "matrix.avg"),
            ("min", "matrix.min"),
            ("max", "matrix.max"),
            ("mode", "matrix.mode"),
            ("trace", "matrix.trace"),
            ("det", "matrix.det"),
            ("rank", "matrix.rank"),
        ] {
            assert_eq!(
                matrix_call_result_builtin_name(method_name),
                Some(builtin_name)
            );
            assert!(
                pine_builtins::get_phase_1_builtin(builtin_name).is_some(),
                "matrix call-result helper `{builtin_name}` must stay registered"
            );
        }

        for method_name in ["size", "set", "fill", "reverse"] {
            assert_eq!(matrix_call_result_builtin_name(method_name), None);
        }
    }

    #[test]
    fn array_call_result_helpers_are_a_closed_registered_set() {
        for (method_name, builtin_name) in [
            ("size", "array.size"),
            ("get", "array.get"),
            ("first", "array.first"),
            ("last", "array.last"),
            ("copy", "array.copy"),
            ("includes", "array.includes"),
            ("indexof", "array.indexof"),
            ("lastindexof", "array.lastindexof"),
            ("binary_search", "array.binary_search"),
            ("binary_search_leftmost", "array.binary_search_leftmost"),
            ("binary_search_rightmost", "array.binary_search_rightmost"),
        ] {
            assert_eq!(
                array_call_result_builtin_name(method_name),
                Some(builtin_name)
            );
            assert!(pine_builtins::get_phase_1_builtin(builtin_name).is_some());
        }
        assert_eq!(array_call_result_builtin_name("min"), None);
        assert_eq!(array_call_result_builtin_name("push"), None);
    }

    #[test]
    fn generic_qualifier_bound_acceptor_drives_expected_label() {
        let accepts = Accepts::QualifierBoundScalar(QualifierBoundScalar::exact(
            Qualifier::Input,
            ScalarKind::Bool,
            true,
        ));
        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "future.input_bool",
            "value",
            accepts,
            pine_type(Qualifier::Const, ValueKind::Bool),
            Span::default(),
        )
        .expect("const bool should not satisfy exact input bool-compatible");

        assert_eq!(
            diagnostic.message,
            "`future.input_bool` argument `value` expects input bool-compatible, got const bool"
        );
    }

    #[test]
    fn acceptor_expected_diagnostic_uses_labels_for_selected_qualifier_bounds() {
        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.sma",
            "length",
            Accepts::SimpleInt,
            pine_type(Qualifier::Series, ValueKind::Int),
            Span::default(),
        )
        .expect("series int should not satisfy simple int");
        assert_eq!(
            diagnostic.message,
            "`ta.sma` argument `length` expects simple int, got series int"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.pivothigh",
            "leftbars",
            Accepts::IntCompatible,
            pine_type(Qualifier::Const, ValueKind::Float),
            Span::default(),
        )
        .expect("const float should not satisfy integer-compatible");
        assert_eq!(
            diagnostic.message,
            "`ta.pivothigh` argument `leftbars` expects integer-compatible, got const float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "hline",
            "price",
            Accepts::AtMostInputNumeric,
            pine_type(Qualifier::Simple, ValueKind::Int),
            Span::default(),
        )
        .expect("simple int should not satisfy at-most-input numeric");
        assert_eq!(
            diagnostic.message,
            "`hline` argument `price` expects const/input numeric, got simple int"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "hline",
            "linewidth",
            Accepts::AtMostInputInt,
            pine_type(Qualifier::Simple, ValueKind::Int),
            Span::default(),
        )
        .expect("simple int should not satisfy at-most-input int");
        assert_eq!(
            diagnostic.message,
            "`hline` argument `linewidth` expects const/input int, got simple int"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "future.input_string",
            "value",
            Accepts::AtMostInputString,
            pine_type(Qualifier::Simple, ValueKind::String),
            Span::default(),
        )
        .expect("simple string should not satisfy at-most-input string");
        assert_eq!(
            diagnostic.message,
            "`future.input_string` argument `value` expects const/input string, got simple string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "future.input_bool",
            "value",
            Accepts::AtMostInputBool,
            pine_type(Qualifier::Series, ValueKind::Bool),
            Span::default(),
        )
        .expect("series bool should not satisfy at-most-input bool");
        assert_eq!(
            diagnostic.message,
            "`future.input_bool` argument `value` expects const/input bool, got series bool"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "future.input_color",
            "value",
            Accepts::AtMostInputColor,
            pine_type(Qualifier::Simple, ValueKind::Color),
            Span::default(),
        )
        .expect("simple color should not satisfy at-most-input color");
        assert_eq!(
            diagnostic.message,
            "`future.input_color` argument `value` expects const/input color, got simple color"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "time",
            "timeframe",
            Accepts::SimpleString,
            pine_type(Qualifier::Const, ValueKind::Int),
            Span::default(),
        )
        .expect("const int should not satisfy simple string");
        assert_eq!(
            diagnostic.message,
            "`time` argument `timeframe` expects simple string, got const int"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "timestamp",
            "dateString",
            Accepts::ConstString,
            pine_type(Qualifier::Simple, ValueKind::String),
            Span::default(),
        )
        .expect("simple string should not satisfy const string");
        assert_eq!(
            diagnostic.message,
            "`timestamp` argument `dateString` expects const string, got simple string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.stdev",
            "biased",
            Accepts::ConstBool,
            pine_type(Qualifier::Series, ValueKind::Bool),
            Span::default(),
        )
        .expect("series bool should not satisfy const bool");
        assert_eq!(
            diagnostic.message,
            "`ta.stdev` argument `biased` expects const bool, got series bool"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "strategy",
            "initial_capital",
            Accepts::ConstNumeric,
            pine_type(Qualifier::Input, ValueKind::Float),
            Span::default(),
        )
        .expect("input float should not satisfy const numeric");
        assert_eq!(
            diagnostic.message,
            "`strategy` argument `initial_capital` expects const numeric, got input float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.alma",
            "offset",
            Accepts::SimpleNumeric,
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy simple numeric");
        assert_eq!(
            diagnostic.message,
            "`ta.alma` argument `offset` expects simple numeric, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.alma",
            "floor",
            Accepts::SimpleBool,
            pine_type(Qualifier::Series, ValueKind::Bool),
            Span::default(),
        )
        .expect("series bool should not satisfy simple bool");
        assert_eq!(
            diagnostic.message,
            "`ta.alma` argument `floor` expects simple bool, got series bool"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.sma",
            "source",
            Accepts::SeriesNumeric,
            pine_type(Qualifier::Const, ValueKind::Int),
            Span::default(),
        )
        .expect("const int should not satisfy series numeric");
        assert_eq!(
            diagnostic.message,
            "`ta.sma` argument `source` expects series numeric, got const int"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.valuewhen",
            "condition",
            Accepts::SeriesNumericOrBool,
            pine_type(Qualifier::Const, ValueKind::String),
            Span::default(),
        )
        .expect("const string should not satisfy series numeric or bool");
        assert_eq!(
            diagnostic.message,
            "`ta.valuewhen` argument `condition` expects series numeric or bool, got const string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.cum",
            "source",
            Accepts::SeriesOrSimpleNumeric,
            pine_type(Qualifier::Const, ValueKind::Bool),
            Span::default(),
        )
        .expect("const bool should not satisfy series/simple numeric");
        assert_eq!(
            diagnostic.message,
            "`ta.cum` argument `source` expects series/simple numeric, got const bool"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "plotshape",
            "series",
            Accepts::SeriesOrSimpleNumericOrBool,
            pine_type(Qualifier::Const, ValueKind::String),
            Span::default(),
        )
        .expect("const string should not satisfy series/simple numeric or bool");
        assert_eq!(
            diagnostic.message,
            "`plotshape` argument `series` expects series/simple numeric or bool, got const string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.bb",
            "mult",
            Accepts::Numeric,
            pine_type(Qualifier::Const, ValueKind::String),
            Span::default(),
        )
        .expect("const string should not satisfy numeric");
        assert_eq!(
            diagnostic.message,
            "`ta.bb` argument `mult` expects numeric, got const string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "timeframe.from_seconds",
            "seconds",
            Accepts::SimpleIntCompatible,
            pine_type(Qualifier::Series, ValueKind::Int),
            Span::default(),
        )
        .expect("series int should not satisfy simple integer-compatible");
        assert_eq!(
            diagnostic.message,
            "`timeframe.from_seconds` argument `seconds` expects simple integer-compatible, got series int"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.barssince",
            "condition",
            Accepts::BoolCompatible,
            pine_type(Qualifier::Const, ValueKind::String),
            Span::default(),
        )
        .expect("const string should not satisfy bool-compatible");
        assert_eq!(
            diagnostic.message,
            "`ta.barssince` argument `condition` expects bool-compatible, got const string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "array.join",
            "separator",
            Accepts::StringCompatible,
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy string-compatible");
        assert_eq!(
            diagnostic.message,
            "`array.join` argument `separator` expects string-compatible, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "str.tostring",
            "value",
            Accepts::StringConvertible,
            pine_type(Qualifier::Simple, ValueKind::ColorArray),
            Span::default(),
        )
        .expect("simple array<color> should not satisfy string-convertible");
        assert_eq!(
            diagnostic.message,
            "`str.tostring` argument `value` expects string-convertible, got simple array<color>"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "int",
            "x",
            Accepts::CastScalar,
            pine_type(Qualifier::Simple, ValueKind::String),
            Span::default(),
        )
        .expect("simple string should not satisfy int/float/bool-compatible");
        assert_eq!(
            diagnostic.message,
            "`int` argument `x` expects int/float/bool-compatible, got simple string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "string",
            "x",
            Accepts::StringCastScalar,
            pine_type(Qualifier::Simple, ValueKind::Color),
            Span::default(),
        )
        .expect("simple color should not satisfy int/float/bool/string-compatible");
        assert_eq!(
            diagnostic.message,
            "`string` argument `x` expects int/float/bool/string-compatible, got simple color"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "fixnan",
            "source",
            Accepts::NumericOrColorCompatible,
            pine_type(Qualifier::Simple, ValueKind::String),
            Span::default(),
        )
        .expect("simple string should not satisfy numeric/color-compatible");
        assert_eq!(
            diagnostic.message,
            "`fixnan` argument `source` expects numeric/color-compatible, got simple string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "array.new_float",
            "initial_value",
            Accepts::NumericCompatible,
            pine_type(Qualifier::Const, ValueKind::String),
            Span::default(),
        )
        .expect("const string should not satisfy numeric-compatible");
        assert_eq!(
            diagnostic.message,
            "`array.new_float` argument `initial_value` expects numeric-compatible, got const string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "input.color",
            "defval",
            Accepts::ColorCompatible,
            pine_type(Qualifier::Const, ValueKind::String),
            Span::default(),
        )
        .expect("const string should not satisfy color-compatible");
        assert_eq!(
            diagnostic.message,
            "`input.color` argument `defval` expects color-compatible, got const string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "array.new_label",
            "initial_value",
            Accepts::LabelCompatible,
            pine_type(Qualifier::Const, ValueKind::String),
            Span::default(),
        )
        .expect("const string should not satisfy label-compatible");
        assert_eq!(
            diagnostic.message,
            "`array.new_label` argument `initial_value` expects label-compatible, got const string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "ta.valuewhen",
            "source",
            Accepts::ValueWhenSource,
            pine_type(Qualifier::Const, ValueKind::String),
            Span::default(),
        )
        .expect("const string should not satisfy valuewhen source");
        assert_eq!(
            diagnostic.message,
            "`ta.valuewhen` argument `source` expects numeric/bool/color-compatible, got const string"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "label.new",
            "size",
            Accepts::StringOrIntCompatible,
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy string/int-compatible");
        assert_eq!(
            diagnostic.message,
            "`label.new` argument `size` expects string/int-compatible, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "label.new",
            "point",
            Accepts::ChartPointCompatible,
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy chart.point-compatible");
        assert_eq!(
            diagnostic.message,
            "`label.new` argument `point` expects chart.point-compatible, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "fill",
            "plot1",
            Accepts::PlotOrHLine,
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy plot/hline");
        assert_eq!(
            diagnostic.message,
            "`fill` argument `plot1` expects plot/hline, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "map.get",
            "id",
            Accepts::Map,
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy map");
        assert_eq!(
            diagnostic.message,
            "`map.get` argument `id` expects map, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "array.size",
            "id",
            Accepts::Array,
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy array");
        assert_eq!(
            diagnostic.message,
            "`array.size` argument `id` expects array, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "array.sum",
            "id",
            Accepts::NumericArray,
            pine_type(Qualifier::Simple, ValueKind::StringArray),
            Span::default(),
        )
        .expect("string array should not satisfy numeric array");
        assert_eq!(
            diagnostic.message,
            "`array.sum` argument `id` expects numeric array, got simple array<string>"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "array.every",
            "id",
            Accepts::NumericOrBoolArray,
            pine_type(Qualifier::Simple, ValueKind::StringArray),
            Span::default(),
        )
        .expect("string array should not satisfy numeric/bool array");
        assert_eq!(
            diagnostic.message,
            "`array.every` argument `id` expects numeric/bool array, got simple array<string>"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "array.sort",
            "id",
            Accepts::NumericOrStringArray,
            pine_type(Qualifier::Simple, ValueKind::BoolArray),
            Span::default(),
        )
        .expect("bool array should not satisfy numeric/string array");
        assert_eq!(
            diagnostic.message,
            "`array.sort` argument `id` expects numeric/string array, got simple array<bool>"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "array.join",
            "id",
            Accepts::ScalarArray,
            pine_type(Qualifier::Simple, ValueKind::ChartPointArray),
            Span::default(),
        )
        .expect("chart.point array should not satisfy scalar array");
        assert_eq!(
            diagnostic.message,
            "`array.join` argument `id` expects scalar array, got simple array<chart.point>"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "matrix.fill",
            "id",
            Accepts::Matrix,
            pine_type(Qualifier::Const, ValueKind::Na),
            Span::default(),
        )
        .expect("const na should not satisfy matrix");
        assert_eq!(
            diagnostic.message,
            "`matrix.fill` argument `id` expects matrix, got const na"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "matrix.sum",
            "id",
            Accepts::NumericMatrix,
            pine_type(Qualifier::Simple, ValueKind::BoolMatrix),
            Span::default(),
        )
        .expect("bool matrix should not satisfy numeric matrix");
        assert_eq!(
            diagnostic.message,
            "`matrix.sum` argument `id` expects numeric matrix, got simple matrix<bool>"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "matrix.mode",
            "id",
            Accepts::FloatMatrix,
            pine_type(Qualifier::Simple, ValueKind::IntMatrix),
            Span::default(),
        )
        .expect("int matrix should not satisfy matrix<float>");
        assert_eq!(
            diagnostic.message,
            "`matrix.mode` argument `id` expects matrix<float>, got simple matrix<int>"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "input.int",
            "options",
            Accepts::Tuple,
            pine_type(Qualifier::Const, ValueKind::Int),
            Span::default(),
        )
        .expect("const int should not satisfy tuple");
        assert_eq!(
            diagnostic.message,
            "`input.int` argument `options` expects tuple, got const int"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "input",
            "defval",
            Accepts::InputDefval,
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy input defval");
        assert_eq!(
            diagnostic.message,
            "`input` argument `defval` expects const int/float/bool/string/color, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "input.int",
            "minval",
            Accepts::Exact(PineType::new(Qualifier::Const, ValueKind::Int)),
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy exact const int");
        assert_eq!(
            diagnostic.message,
            "`input.int` argument `minval` expects const int, got series float"
        );

        let diagnostic = call_arg_accepts_type_expected_diagnostic(
            "future.kind",
            "value",
            Accepts::Kind(ValueKind::Plot),
            pine_type(Qualifier::Series, ValueKind::Float),
            Span::default(),
        )
        .expect("series float should not satisfy plot kind");
        assert_eq!(
            diagnostic.message,
            "`future.kind` argument `value` expects plot, got series float"
        );
    }

    #[test]
    fn acceptor_expected_diagnostic_returns_none_for_valid_arguments() {
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "ta.highest",
                "length",
                Accepts::SimpleInt,
                pine_type(Qualifier::Input, ValueKind::Int),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "ta.pivothigh",
                "rightbars",
                Accepts::IntCompatible,
                pine_type(Qualifier::Series, ValueKind::Int),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "hline",
                "price",
                Accepts::AtMostInputNumeric,
                pine_type(Qualifier::Input, ValueKind::Float),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "hline",
                "linewidth",
                Accepts::AtMostInputInt,
                pine_type(Qualifier::Input, ValueKind::Int),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "future.input_string",
                "value",
                Accepts::AtMostInputString,
                pine_type(Qualifier::Input, ValueKind::String),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "future.input_bool",
                "value",
                Accepts::AtMostInputBool,
                pine_type(Qualifier::Const, ValueKind::Bool),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "future.input_color",
                "value",
                Accepts::AtMostInputColor,
                pine_type(Qualifier::Input, ValueKind::Color),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "time",
                "timeframe",
                Accepts::SimpleString,
                pine_type(Qualifier::Simple, ValueKind::String),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "timestamp",
                "dateString",
                Accepts::ConstString,
                pine_type(Qualifier::Const, ValueKind::String),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "ta.stdev",
                "biased",
                Accepts::ConstBool,
                pine_type(Qualifier::Const, ValueKind::Bool),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "strategy",
                "initial_capital",
                Accepts::ConstNumeric,
                pine_type(Qualifier::Const, ValueKind::Float),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "ta.alma",
                "offset",
                Accepts::SimpleNumeric,
                pine_type(Qualifier::Simple, ValueKind::Float),
                Span::default(),
            )
            .is_none()
        );
        assert!(
            call_arg_accepts_type_expected_diagnostic(
                "ta.alma",
                "floor",
                Accepts::SimpleBool,
                pine_type(Qualifier::Input, ValueKind::Bool),
                Span::default(),
            )
            .is_none()
        );
    }
}
