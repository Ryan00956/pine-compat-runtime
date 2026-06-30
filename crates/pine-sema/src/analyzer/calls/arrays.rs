use crate::analyzer::user_types::UserTypeArrayElementInference;
use crate::prelude::*;

impl Analyzer {
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
        let Some(element_kind) = array_type.kind.array_element_kind() else {
            return;
        };
        if value_type.kind == ValueKind::Na
            || value_type.kind == element_kind
            || (element_kind == ValueKind::Float && value_type.kind == ValueKind::Int)
        {
            return;
        }
        let expected = match element_kind {
            ValueKind::Float => "float arrays",
            ValueKind::Int => "int arrays",
            ValueKind::Bool => "bool arrays",
            ValueKind::String => "string arrays",
            ValueKind::Color => "color arrays",
            ValueKind::Label => "label arrays",
            ValueKind::Line => "line arrays",
            ValueKind::LineFill => "linefill arrays",
            ValueKind::Polyline => "polyline arrays",
            ValueKind::Box => "box arrays",
            ValueKind::Table => "table arrays",
            ValueKind::ChartPoint => "chart.point arrays",
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
        match self.array_from_user_type_element_inference(args, arg_types) {
            Some(UserTypeArrayElementInference::SameScalarLocal(_)) => return,
            Some(_) => {
                self.diagnostics.push(Diagnostic::error(
                    "E_CALL_ARG_TYPE",
                    "`array.from` does not support UDT arrays",
                    args.first().map_or(Span::default(), |arg| arg.span),
                ));
                return;
            }
            None => {}
        }

        self.diagnostics.push(Diagnostic::error(
            "E_CALL_ARG_TYPE",
            "`array.from` arguments must infer one supported array element kind",
            args.first().map_or(Span::default(), |arg| arg.span),
        ));
    }
}
