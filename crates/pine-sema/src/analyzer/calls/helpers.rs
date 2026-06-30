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
            | "matrix.add_row"
            | "matrix.add_col"
            | "matrix.remove_col"
            | "matrix.remove_row"
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
