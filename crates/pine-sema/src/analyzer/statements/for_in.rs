use crate::prelude::*;

const FOR_IN_SUPPORTED_ITERABLES_REASON: &str = "for...in currently supports statement iteration over scalar arrays, label arrays, line arrays, linefill arrays, polyline arrays, box arrays, table arrays, chart.point arrays, same-local or same-imported scalar-field UDT arrays, and matrix rows only; non-scalar-field UDT arrays, map, and other iterable families remain unsupported";

const FOR_IN_SUPPORTED_INDEX_VALUE_REASON: &str = "index/value for...in currently supports statement iteration over array<int>, array<float>, array<bool>, array<string>, array<color>, array<label>, array<line>, array<linefill>, array<polyline>, array<box>, array<table>, array<chart.point>, same-local or same-imported scalar-field UDT arrays, and matrix rows only; other array element kinds, non-scalar-field UDT arrays, map, and expression-form for...in remain unsupported";

impl Analyzer {
    pub(super) fn analyze_for_in_stmt(
        &mut self,
        index: Option<&str>,
        value: &str,
        iterable: &Expr,
        body: &[Stmt],
        span: Span,
    ) {
        let iterable_type = self.analyze_expr(iterable);
        let Some(iterable_type) = iterable_type else {
            self.unsupported("for...in", FOR_IN_SUPPORTED_ITERABLES_REASON, span);
            return;
        };
        if index.is_some() && !supports_index_value_for_in(iterable_type.kind) {
            self.unsupported("for...in", FOR_IN_SUPPORTED_INDEX_VALUE_REASON, span);
            return;
        }
        let Some((value_kind, user_type_name)) =
            for_in_loop_value_kind(iterable_type.kind, self, iterable)
        else {
            self.unsupported("for...in", FOR_IN_SUPPORTED_ITERABLES_REASON, span);
            return;
        };
        self.compatibility.supported.push(FeatureUse {
            feature: "for".to_owned(),
            span,
        });

        self.block_depth += 1;
        self.loop_depth += 1;
        self.scope.push_scope();
        if let Some(index) = index {
            let index_symbol = self.define_local_symbol(
                index,
                PineType::new(Qualifier::Series, ValueKind::Int),
                None,
                self.function_depth == 0,
            );
            self.bind_symbol(index, span, index_symbol);
        }
        let value_symbol = self.define_local_symbol(
            value,
            PineType::new(Qualifier::Series, value_kind),
            None,
            self.function_depth == 0,
        );
        self.bind_symbol(value, span, value_symbol);
        if let Some(type_name) = user_type_name {
            self.mark_symbol_id_user_type(value_symbol.id, type_name);
        }
        for body_statement in body {
            self.analyze_stmt(body_statement);
        }
        self.scope.pop_scope();
        self.loop_depth -= 1;
        self.block_depth -= 1;
    }
}

fn supports_index_value_for_in(iterable_kind: ValueKind) -> bool {
    matches!(
        iterable_kind,
        ValueKind::IntArray
            | ValueKind::FloatArray
            | ValueKind::BoolArray
            | ValueKind::StringArray
            | ValueKind::ColorArray
            | ValueKind::LabelArray
            | ValueKind::LineArray
            | ValueKind::LineFillArray
            | ValueKind::PolylineArray
            | ValueKind::BoxArray
            | ValueKind::TableArray
            | ValueKind::ChartPointArray
            | ValueKind::UserTypeArray
            | ValueKind::FloatMatrix
            | ValueKind::IntMatrix
            | ValueKind::BoolMatrix
            | ValueKind::StringMatrix
            | ValueKind::ColorMatrix
    )
}

fn for_in_loop_value_kind(
    iterable_kind: ValueKind,
    analyzer: &Analyzer,
    iterable: &Expr,
) -> Option<(ValueKind, Option<String>)> {
    match iterable_kind {
        ValueKind::IntArray => Some((ValueKind::Int, None)),
        ValueKind::FloatArray => Some((ValueKind::Float, None)),
        ValueKind::BoolArray => Some((ValueKind::Bool, None)),
        ValueKind::StringArray => Some((ValueKind::String, None)),
        ValueKind::ColorArray => Some((ValueKind::Color, None)),
        ValueKind::LabelArray => Some((ValueKind::Label, None)),
        ValueKind::LineArray => Some((ValueKind::Line, None)),
        ValueKind::LineFillArray => Some((ValueKind::LineFill, None)),
        ValueKind::PolylineArray => Some((ValueKind::Polyline, None)),
        ValueKind::BoxArray => Some((ValueKind::Box, None)),
        ValueKind::TableArray => Some((ValueKind::Table, None)),
        ValueKind::ChartPointArray => Some((ValueKind::ChartPoint, None)),
        ValueKind::FloatMatrix => Some((ValueKind::FloatArray, None)),
        ValueKind::IntMatrix => Some((ValueKind::IntArray, None)),
        ValueKind::BoolMatrix => Some((ValueKind::BoolArray, None)),
        ValueKind::StringMatrix => Some((ValueKind::StringArray, None)),
        ValueKind::ColorMatrix => Some((ValueKind::ColorArray, None)),
        ValueKind::UserTypeArray => analyzer
            .user_type_array_name_of_expr(iterable)
            .map(|type_name| (ValueKind::UserType, Some(type_name))),
        _ => None,
    }
}
