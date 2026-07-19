use crate::prelude::*;

impl Analyzer {
    pub(crate) fn resolve_qualified_value(&mut self, name: &str, span: Span) -> Option<PineType> {
        if self.validate_strategy_state_variable(name, span) {
            return None;
        }
        if pine_builtins::named_color(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::Color));
        }
        if let Some(pine_type) = pine_builtins::builtin_series_value_type(name) {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(pine_type);
        }
        if pine_builtins::named_float_constant(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::Float));
        }
        if pine_builtins::named_int_constant(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::Int));
        }
        if pine_builtins::named_string_constant(name).is_some() {
            self.compatibility.supported.push(FeatureUse {
                feature: name.to_owned(),
                span,
            });
            return Some(PineType::new(Qualifier::Const, ValueKind::String));
        }

        self.check_feature_name(name, span);
        if name.starts_with("color.") {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_COLOR",
                format!("unknown named color `{name}`"),
                span,
            ));
        }
        None
    }

    pub(crate) fn resolve_symbol(&mut self, name: &str, span: Span) -> Option<PineType> {
        if let Some(symbol) = self.scope.resolve(name) {
            self.bind_symbol(name, span, symbol);
            Some(symbol.pine_type)
        } else if let Some(resolution) = self.legacy.resolve_value(name) {
            match resolution {
                crate::legacy::LegacyResolution::ExactAlias(rule) => {
                    let canonical_name = rule
                        .canonical_name
                        .expect("validated exact legacy alias has a canonical target");
                    let pine_type = self
                        .resolve_qualified_value(canonical_name, span)
                        .expect("validated exact legacy symbol target is registered");
                    let source_context_id = self.current_source_context_id();
                    self.legacy.record_value_translation(
                        &mut self.compatibility,
                        source_context_id,
                        span,
                        rule,
                    );
                    self.expr_types.insert(self.expr_key(span), pine_type);
                    Some(pine_type)
                }
                crate::legacy::LegacyResolution::UnsupportedKnown(rule) => {
                    let crate::legacy::LegacyRuleSupport::UnsupportedKnown { reason } =
                        rule.support
                    else {
                        unreachable!("legacy resolver preserves rule support state")
                    };
                    self.unsupported(rule.source_name, reason, span);
                    None
                }
            }
        } else {
            self.diagnostics.push(Diagnostic::error(
                "E_UNKNOWN_SYMBOL",
                format!("unknown symbol `{name}`"),
                span,
            ));
            None
        }
    }
}
