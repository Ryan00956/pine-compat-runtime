use crate::prelude::*;

pub(super) enum FocusedLegacyCallAnalysis {
    NotApplicable,
    Analyzed(Option<PineType>),
}

impl Analyzer {
    pub(super) fn analyze_focused_legacy_call(
        &mut self,
        name: &str,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let Some(resolution) = self.legacy.resolve_call(name) else {
            return FocusedLegacyCallAnalysis::NotApplicable;
        };
        match resolution {
            crate::legacy::LegacyResolution::Focused(rule) => match rule.kind {
                crate::legacy::LegacyRuleKind::FocusedInput if name == "input" => {
                    self.analyze_focused_legacy_input(rule, callee_span, call_span, args, arg_types)
                }
                crate::legacy::LegacyRuleKind::FocusedOutput => self.analyze_focused_legacy_output(
                    rule,
                    name,
                    callee_span,
                    call_span,
                    args,
                    arg_types,
                ),
                crate::legacy::LegacyRuleKind::FocusedSecurity if name == "security" => self
                    .analyze_focused_legacy_security(rule, callee_span, call_span, args, arg_types),
                crate::legacy::LegacyRuleKind::FocusedExpression
                | crate::legacy::LegacyRuleKind::FocusedCall => self
                    .analyze_focused_legacy_expression(
                        rule,
                        name,
                        callee_span,
                        call_span,
                        args,
                        arg_types,
                    ),
                _ => FocusedLegacyCallAnalysis::NotApplicable,
            },
            crate::legacy::LegacyResolution::UnsupportedKnown(rule)
                if matches!(
                    rule.kind,
                    crate::legacy::LegacyRuleKind::FocusedInput
                        | crate::legacy::LegacyRuleKind::FocusedOutput
                        | crate::legacy::LegacyRuleKind::FocusedExpression
                        | crate::legacy::LegacyRuleKind::FocusedCall
                        | crate::legacy::LegacyRuleKind::FocusedSecurity
                ) =>
            {
                let crate::legacy::LegacyRuleSupport::UnsupportedKnown { reason } = rule.support
                else {
                    unreachable!("legacy resolver preserves rule support state")
                };
                self.unsupported(rule.source_name, reason, callee_span);
                FocusedLegacyCallAnalysis::Analyzed(None)
            }
            _ => FocusedLegacyCallAnalysis::NotApplicable,
        }
    }

    fn analyze_focused_legacy_expression(
        &mut self,
        rule: crate::legacy::LegacyRule,
        name: &str,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let bound = match self
            .legacy
            .bind_legacy_expression(name, args, arg_types, call_span)
        {
            crate::legacy::LegacyExpressionBinding::Bound(bound) => bound,
            crate::legacy::LegacyExpressionBinding::Invalid(diagnostics) => {
                self.diagnostics.extend(diagnostics);
                return FocusedLegacyCallAnalysis::Analyzed(None);
            }
        };
        let pine_type = match bound.kind {
            crate::legacy::LegacyExpressionKind::Iff => self.analyze_legacy_iff(&bound, call_span),
            crate::legacy::LegacyExpressionKind::Offset => self.analyze_legacy_offset(&bound),
            crate::legacy::LegacyExpressionKind::RsiLength => {
                let mut canonical_args = bound.ordered_args.clone();
                canonical_args[0].name = Some("source".to_owned());
                canonical_args[1].name = Some("length".to_owned());
                let signature = pine_builtins::get_phase_1_builtin("ta.rsi")
                    .expect("canonical RSI signature is registered");
                self.analyze_registered_builtin(
                    "ta.rsi",
                    signature,
                    callee_span,
                    call_span,
                    &canonical_args,
                    &bound
                        .ordered_arg_types
                        .iter()
                        .copied()
                        .map(Some)
                        .collect::<Vec<_>>(),
                )
            }
            crate::legacy::LegacyExpressionKind::RsiSeries => {
                self.analyze_legacy_rsi_series(&bound, call_span)
            }
        };
        let Some(pine_type) = pine_type else {
            return FocusedLegacyCallAnalysis::Analyzed(None);
        };
        let source_context_id = self.current_source_context_id();
        self.legacy.record_expression_translation(
            &mut self.compatibility,
            source_context_id,
            callee_span,
            rule,
            &bound,
        );
        FocusedLegacyCallAnalysis::Analyzed(Some(pine_type))
    }

    fn analyze_focused_legacy_security(
        &mut self,
        rule: crate::legacy::LegacyRule,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let const_strings = args
            .iter()
            .map(|arg| self.known_const_string_value(&arg.value))
            .collect::<Vec<_>>();
        let const_bools = args
            .iter()
            .map(|arg| self.known_const_bool_value(&arg.value))
            .collect::<Vec<_>>();
        let bound = match self.legacy.bind_legacy_security_args(
            args,
            arg_types,
            &const_strings,
            &const_bools,
            call_span,
        ) {
            crate::legacy::LegacySecurityBinding::Bound(bound) => bound,
            crate::legacy::LegacySecurityBinding::Invalid(diagnostics) => {
                self.diagnostics.extend(diagnostics);
                return FocusedLegacyCallAnalysis::Analyzed(None);
            }
        };
        let unsupported_before = self.compatibility.unsupported.len();
        let pine_type = self.analyze_bound_legacy_security(callee_span, &bound);
        if self.compatibility.unsupported.len() != unsupported_before || pine_type.is_none() {
            return FocusedLegacyCallAnalysis::Analyzed(pine_type);
        }
        let source_context_id = self.current_source_context_id();
        self.legacy.record_security_translation(
            &mut self.compatibility,
            source_context_id,
            callee_span,
            call_span,
            rule,
            &bound,
        );
        FocusedLegacyCallAnalysis::Analyzed(pine_type)
    }

    fn analyze_legacy_iff(
        &mut self,
        bound: &crate::legacy::BoundLegacyExpression,
        call_span: Span,
    ) -> Option<PineType> {
        let [condition_type, then_type, else_type] = bound.ordered_arg_types.as_slice() else {
            unreachable!("validated iff arity")
        };
        self.expect_bool(*condition_type, bound.ordered_args[0].span);
        if !matches!(
            then_type.kind,
            ValueKind::Int
                | ValueKind::Float
                | ValueKind::Bool
                | ValueKind::String
                | ValueKind::Color
                | ValueKind::Na
        ) || !matches!(
            else_type.kind,
            ValueKind::Int
                | ValueKind::Float
                | ValueKind::Bool
                | ValueKind::String
                | ValueKind::Color
                | ValueKind::Na
        ) {
            self.unsupported(
                "iff.result",
                "the current legacy iff slice supports scalar numeric, bool, string, color, and na results",
                call_span,
            );
            return None;
        }
        self.merge_branch_types(
            *condition_type,
            *then_type,
            *else_type,
            self.known_const_bool_value(&bound.ordered_args[0].value),
            call_span,
        )
    }

    fn analyze_legacy_offset(
        &mut self,
        bound: &crate::legacy::BoundLegacyExpression,
    ) -> Option<PineType> {
        let source_type = bound.ordered_arg_types[0];
        if matches!(source_type.kind, ValueKind::Tuple | ValueKind::Void) {
            self.unsupported(
                "offset.source",
                "legacy offset does not support tuple or void source values",
                bound.ordered_args[0].span,
            );
            return None;
        }
        self.validate_history_offset(
            &bound.ordered_args[1].value,
            Some(bound.ordered_arg_types[1]),
        );
        Some(PineType::new(Qualifier::Series, source_type.kind))
    }

    fn analyze_legacy_rsi_series(
        &mut self,
        bound: &crate::legacy::BoundLegacyExpression,
        call_span: Span,
    ) -> Option<PineType> {
        let first = bound.ordered_arg_types[0];
        let second = bound.ordered_arg_types[1];
        if first.qualifier != Qualifier::Series
            || !matches!(first.kind, ValueKind::Int | ValueKind::Float)
            || !matches!(second.kind, ValueKind::Int | ValueKind::Float)
        {
            self.diagnostics.push(Diagnostic::error(
                "E_LEGACY_RSI_OVERLOAD",
                "Pine v4 two-series `rsi(x, y)` requires a series numeric first argument and a numeric second argument",
                call_span,
            ));
            return None;
        }
        Some(PineType::new(Qualifier::Series, ValueKind::Float))
    }

    fn analyze_focused_legacy_input(
        &mut self,
        rule: crate::legacy::LegacyRule,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let explicit_type_marker = self
            .legacy
            .explicit_v4_input_type_expr(args)
            .and_then(|expr| self.known_const_string_value(expr));
        let bound =
            match self
                .legacy
                .bind_v4_input_args(args, arg_types, explicit_type_marker.as_deref())
            {
                crate::legacy::LegacyInputBinding::Bound(bound) => bound,
                crate::legacy::LegacyInputBinding::Invalid(diagnostics) => {
                    self.diagnostics.extend(diagnostics);
                    return FocusedLegacyCallAnalysis::Analyzed(None);
                }
            };
        let signature = pine_builtins::get_phase_1_builtin(bound.canonical_name)
            .expect("validated focused input target is registered");
        let source_context_id = self.current_source_context_id();
        self.legacy.record_input_translation(
            &mut self.compatibility,
            source_context_id,
            callee_span,
            rule,
            bound.canonical_name,
            bound.arg_rewrites,
        );
        FocusedLegacyCallAnalysis::Analyzed(self.analyze_registered_builtin(
            bound.canonical_name,
            signature,
            callee_span,
            call_span,
            &bound.canonical_args,
            &bound.canonical_arg_types,
        ))
    }

    fn analyze_focused_legacy_output(
        &mut self,
        rule: crate::legacy::LegacyRule,
        name: &str,
        callee_span: Span,
        call_span: Span,
        args: &[CallArg],
        arg_types: &[Option<PineType>],
    ) -> FocusedLegacyCallAnalysis {
        let const_strings = args
            .iter()
            .map(|arg| self.known_const_string_value(&arg.value))
            .collect::<Vec<_>>();
        let const_ints = args
            .iter()
            .map(|arg| self.known_const_int_value(&arg.value))
            .collect::<Vec<_>>();
        let bound = match self.legacy.bind_v4_output_args(
            name,
            args,
            arg_types,
            &const_strings,
            &const_ints,
        ) {
            crate::legacy::LegacyOutputBinding::Bound(bound) => bound,
            crate::legacy::LegacyOutputBinding::Invalid(diagnostics) => {
                self.diagnostics.extend(diagnostics);
                return FocusedLegacyCallAnalysis::Analyzed(None);
            }
        };
        let signature = pine_builtins::get_phase_1_builtin(bound.canonical_name)
            .expect("validated focused output target is registered");
        let source_context_id = self.current_source_context_id();
        self.legacy.record_output_translation(
            &mut self.compatibility,
            source_context_id,
            callee_span,
            rule,
            crate::legacy::LegacyOutputTranslationPlan {
                canonical_name: bound.canonical_name,
                arg_rewrites: bound.arg_rewrites,
                requires_adaptation: bound.requires_adaptation,
                emulates_transparency: bound.emulates_transparency,
                emulates_numeric_style: bound.emulates_numeric_style,
            },
        );
        FocusedLegacyCallAnalysis::Analyzed(self.analyze_registered_builtin(
            bound.canonical_name,
            signature,
            callee_span,
            call_span,
            &bound.canonical_args,
            &bound.canonical_arg_types,
        ))
    }
}
