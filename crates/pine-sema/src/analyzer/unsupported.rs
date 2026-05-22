use crate::prelude::*;

pub(crate) const VARIP_UNSUPPORTED_REASON: &str = "varip intrabar persistence is not implemented; forming-bar rollback currently supports var state only";

pub(crate) fn unsupported_syntax_reason(feature: &str) -> &'static str {
    match feature {
        "import" => "library imports are not supported in Phase 1",
        "function" => "unsupported user-defined function syntax",
        "for" => "unsupported for loop syntax",
        _ => "syntax is not supported in Phase 1",
    }
}

impl Analyzer {
    pub(crate) fn check_feature_expr(&mut self, expr: &Expr) {
        let Some(name) = expr_name(expr) else {
            return;
        };
        self.check_feature_name(&name, expr.span);
    }

    pub(crate) fn check_feature_name(&mut self, name: &str, span: Span) {
        let unsupported_reason = if pine_builtins::is_phase_1_builtin(name) {
            None
        } else if name.starts_with("strategy.") {
            Some("strategy backtesting and broker emulation are outside the current runtime scope")
        } else if name.starts_with("request.") {
            Some("multi-symbol and multi-timeframe data requests are not supported in Phase 1")
        } else if name.starts_with("array.") {
            Some("this array function is not supported in the current partial array subset")
        } else if matches!(name, "alert" | "alertcondition") {
            Some("alerts are not supported in Phase 1")
        } else if name.starts_with("label.")
            || name.starts_with("line.")
            || name.starts_with("box.")
            || name.starts_with("table.")
            || name.starts_with("polyline.")
        {
            Some("this drawing object call is not supported in the current partial drawing subset")
        } else {
            None
        };

        if let Some(reason) = unsupported_reason {
            self.unsupported(name, reason, span);
        } else if pine_builtins::is_phase_1_builtin(name) {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
        }
    }

    pub(crate) fn unsupported(&mut self, feature: &str, reason: &str, span: Span) {
        self.compatibility.unsupported.push(UnsupportedFeature {
            feature: feature.to_owned(),
            reason: reason.to_owned(),
            span,
        });
        self.diagnostics.push(Diagnostic {
            code: "E_UNSUPPORTED_FEATURE".to_owned(),
            severity: Severity::Error,
            message: format!("`{feature}` is not supported: {reason}"),
            span,
        });
    }
}
