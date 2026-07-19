use crate::prelude::*;

impl Analyzer {
    pub(super) fn lower_legacy_value(&self, span: Span) -> Option<HirExprKind> {
        if let Some(value) = self
            .legacy
            .canonical_string_value(self.current_source_context_id(), span)
        {
            return Some(HirExprKind::Literal(HirLiteral::String(value.to_owned())));
        }
        self.legacy
            .canonical_value_name(self.current_source_context_id(), span)
            .map(|canonical_name| HirExprKind::Builtin(canonical_name.to_owned()))
    }

    pub(super) fn lower_legacy_call_args(
        &self,
        span: Span,
        args: &[CallArg],
    ) -> Option<Vec<CallArg>> {
        self.legacy
            .canonical_call_arg_rewrites(self.current_source_context_id(), span)
            .map(|rewrites| {
                args.iter()
                    .zip(rewrites)
                    .filter_map(|(arg, rewrite)| {
                        if !rewrite.keep {
                            return None;
                        }
                        let mut arg = arg.clone();
                        arg.name = rewrite.canonical_name.map(str::to_owned);
                        Some(arg)
                    })
                    .collect()
            })
    }

    pub(super) fn lower_legacy_history_offset(
        &mut self,
        span: Span,
        args: &[CallArg],
        pine_type: PineType,
        series_id: Option<pine_ir::SeriesId>,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        let canonical_args = self
            .lower_legacy_call_args(span, args)
            .expect("focused legacy history lowering records argument roles");
        let source = canonical_args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("source"))
            .expect("validated legacy offset source");
        let offset_arg = canonical_args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("offset"))
            .expect("validated legacy offset amount");
        let offset = match self
            .known_history_offset_int_value(&offset_arg.value)
            .and_then(|value| u32::try_from(value).ok())
        {
            Some(offset) => HirHistoryOffset::Constant(offset),
            None => HirHistoryOffset::Dynamic(Box::new(self.lower_expr_with_params(
                &offset_arg.value,
                param_exprs,
                param_types,
            )?)),
        };
        let mut lowered_source =
            self.lower_expr_with_params(&source.value, param_exprs, param_types)?;
        if lowered_source.series_id.is_none() {
            lowered_source.series_id = self.lower_expr_series_id(
                &source.value,
                PineType::new(Qualifier::Series, lowered_source.pine_type.kind),
            );
        }
        Some(HirExpr {
            pine_type,
            series_id,
            kind: HirExprKind::History {
                expr: Box::new(lowered_source),
                offset,
            },
        })
    }
}
