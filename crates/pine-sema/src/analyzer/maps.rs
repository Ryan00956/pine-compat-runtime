use crate::prelude::*;

impl Analyzer {
    pub(crate) fn map_type_of_expr(&self, expr: &Expr) -> Option<MapTypeInfo> {
        if let Some(info) = self.expr_maps.get(&span_key(expr.span)).copied() {
            return Some(info);
        }

        match &expr.kind {
            ExprKind::Identifier(name) => self
                .scope
                .resolve(name)
                .and_then(|symbol| self.symbol_maps.get(&symbol.id).copied()),
            ExprKind::QualifiedName(parts) if parts.len() == 1 => self
                .scope
                .resolve(&parts[0])
                .and_then(|symbol| self.symbol_maps.get(&symbol.id).copied()),
            _ => None,
        }
    }

    pub(crate) fn mark_expr_map(&mut self, span: Span, info: MapTypeInfo) {
        self.expr_maps.insert(span_key(span), info);
    }

    pub(crate) fn mark_symbol_map(&mut self, symbol: SymbolInfo, info: MapTypeInfo) {
        self.symbol_maps.insert(symbol.id, info);
    }

    pub(crate) fn validate_map_value_assignment(
        &mut self,
        name: &str,
        target_info: MapTypeInfo,
        value: &Expr,
        value_type: PineType,
        span: Span,
    ) {
        if value_type.kind == ValueKind::Na {
            return;
        }
        if self
            .map_type_of_expr(value)
            .is_some_and(|value_info| value_info == target_info)
        {
            return;
        }
        self.diagnostics.push(Diagnostic::error(
            "E_MAP_ASSIGN_TYPE",
            format!("cannot assign a different map template to `{name}`"),
            span,
        ));
    }
}

pub(crate) fn map_kind_from_template_name(name: &str) -> Option<ValueKind> {
    match name {
        "int" => Some(ValueKind::Int),
        "float" => Some(ValueKind::Float),
        "bool" => Some(ValueKind::Bool),
        "string" => Some(ValueKind::String),
        "color" => Some(ValueKind::Color),
        _ => None,
    }
}

pub(crate) fn accepts_map_scalar_kind(expected: ValueKind, actual: PineType) -> bool {
    let kind_matches = match expected {
        ValueKind::Float => matches!(
            actual.kind,
            ValueKind::Float | ValueKind::Int | ValueKind::Na
        ),
        ValueKind::Int => matches!(actual.kind, ValueKind::Int | ValueKind::Na),
        ValueKind::Bool => matches!(actual.kind, ValueKind::Bool | ValueKind::Na),
        ValueKind::String => matches!(actual.kind, ValueKind::String | ValueKind::Na),
        ValueKind::Color => matches!(actual.kind, ValueKind::Color | ValueKind::Na),
        _ => false,
    };
    kind_matches && qualifier_at_most(actual.qualifier, Qualifier::Series)
}
