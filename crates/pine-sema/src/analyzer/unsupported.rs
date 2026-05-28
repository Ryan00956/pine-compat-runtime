use crate::prelude::*;

pub(crate) const VARIP_DRAWING_UNSUPPORTED_REASON: &str = "varip drawing object ids are not supported; retaining only an id would be unsafe while drawing object stores roll back between forming updates";
pub(crate) const VARIP_VALUE_UNSUPPORTED_REASON: &str = "varip currently supports scalar int, float, bool, string, color, na, and scalar typed-array declarations only; drawing ids, tuples, UDTs, and other value families are not implemented";
pub(crate) const STRATEGY_UNSUPPORTED_REASON: &str = "strategy order functions beyond the current strategy.entry subset, broker emulation, and backtesting are outside the current runtime scope";
pub(crate) const STRATEGY_STATE_UNSUPPORTED_REASON: &str = "strategy state variables are not supported until the Phase L strategy usability subset is implemented";

pub(crate) fn unsupported_strategy_reason(name: &str) -> Option<&'static str> {
    if name == "strategy" || name.starts_with("strategy.") {
        Some(STRATEGY_UNSUPPORTED_REASON)
    } else {
        None
    }
}

pub(crate) fn unsupported_syntax_reason(feature: &str) -> &'static str {
    match feature {
        "import" => "this import form is outside the supported Phase J subset",
        "library" => "library declarations are not supported in Phase J Slice 0",
        "export" => "export declarations are not supported in Phase J Slice 0",
        "user-defined types" => "user-defined types are not supported in Phase J Slice 0",
        "user-defined methods" => "user-defined methods are not supported in Phase J Slice 0",
        "user-defined type field mutation" => {
            "user-defined type field mutation is not supported; Phase J UDT values are immutable in the current subset"
        }
        "strategy state variable mutation" => {
            "strategy state variables are read-only in the Phase L strategy subset"
        }
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
            Some(STRATEGY_UNSUPPORTED_REASON)
        } else if name.starts_with("request.") {
            Some("multi-symbol and multi-timeframe data requests are not supported in Phase 1")
        } else if name.starts_with("array.") {
            Some("this array function is not supported in the current partial array subset")
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
