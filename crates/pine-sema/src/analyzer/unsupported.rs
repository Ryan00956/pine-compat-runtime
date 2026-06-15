use crate::prelude::*;

pub(crate) const VARIP_DRAWING_UNSUPPORTED_REASON: &str = "varip drawing object ids are not supported; retaining only an id would be unsafe while drawing object stores roll back between forming updates";
pub(crate) const VARIP_VALUE_UNSUPPORTED_REASON: &str = "varip currently supports scalar int, float, bool, string, color, na, and scalar typed-array declarations only; drawing ids, tuples, UDTs, and other value families are not implemented";
pub(crate) const LOG_UNSUPPORTED_REASON: &str = "Pine Logs output is not implemented; log.info, log.warning, and log.error require a host-owned log pane/output contract";
pub(crate) const STRATEGY_UNSUPPORTED_REASON: &str = "strategy order functions beyond the supported strategy.entry/strategy.close/strategy.close_all/strategy.cancel/strategy.cancel_all/strategy.exit subset, broker emulation settings, and rich backtesting features are not implemented";
pub(crate) const STRATEGY_STATE_UNSUPPORTED_REASON: &str = "strategy state variables beyond the supported position, profit, equity, and trade-count subset are not implemented";

pub(crate) fn unsupported_strategy_reason(name: &str) -> Option<&'static str> {
    if name == "strategy" || name.starts_with("strategy.") {
        Some(STRATEGY_UNSUPPORTED_REASON)
    } else {
        None
    }
}

pub(crate) fn unsupported_log_reason(name: &str) -> Option<&'static str> {
    if matches!(name, "log.info" | "log.warning" | "log.error") {
        Some(LOG_UNSUPPORTED_REASON)
    } else {
        None
    }
}

pub(crate) fn unsupported_syntax_reason(feature: &str) -> &'static str {
    match feature {
        "import" => "this import form is outside the supported host-provided library import subset",
        "library" => "library declarations are not supported in executable scripts",
        "export" => "export declarations are only supported in host-provided library sources",
        "user-defined types" => {
            "this user-defined type form is outside the supported local scalar-field UDT subset"
        }
        "user-defined methods" => {
            "this user-defined method form is outside the supported pure local UDT method subset"
        }
        "user-defined type field mutation" => {
            "user-defined type field mutation is not supported; UDT values are immutable in the current subset"
        }
        "strategy state variable mutation" => {
            "strategy state variables are read-only in the current strategy subset"
        }
        "function" => "unsupported user-defined function syntax",
        "for" => "unsupported for loop syntax",
        _ => "syntax is not supported in the current language subset",
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
        } else if let Some(reason) = unsupported_log_reason(name) {
            Some(reason)
        } else if name.starts_with("strategy.") {
            Some(STRATEGY_UNSUPPORTED_REASON)
        } else if name.starts_with("request.") {
            Some("this request function is outside the supported request.security subset")
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
