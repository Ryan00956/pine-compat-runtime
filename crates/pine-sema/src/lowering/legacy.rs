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
}
