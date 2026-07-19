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
                _ => FocusedLegacyCallAnalysis::NotApplicable,
            },
            crate::legacy::LegacyResolution::UnsupportedKnown(rule)
                if matches!(
                    rule.kind,
                    crate::legacy::LegacyRuleKind::FocusedInput
                        | crate::legacy::LegacyRuleKind::FocusedOutput
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
