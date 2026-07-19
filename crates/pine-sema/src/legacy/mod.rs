mod calls;
mod catalog;
mod declarations;
mod dialect;
mod expressions;
mod inputs;
mod lowering;
mod outputs;
mod report;
mod resolver;
mod security;

use pine_syntax::{Program, Span};

pub use catalog::{
    LEGACY_RULES, LEGACY_TRANSLATOR_REVISION, LegacyRule, LegacyRuleKind, LegacyRuleSupport,
};
pub use declarations::ScriptModeClassification;
pub use dialect::{
    MAX_PINE_LANGUAGE_VERSION, MIN_PINE_LANGUAGE_VERSION, PineDialect, VersionOrigin,
};

#[cfg(test)]
pub(crate) use catalog::{CatalogValidationError, validate_catalog};
pub(crate) use declarations::{LegacyAdmissionFailure, legacy_admission_failure};
pub(crate) use dialect::LanguageSelection;
pub(crate) use report::normalize_legacy_report;
pub(crate) use resolver::LegacyResolution;

use crate::compatibility::CompatibilityReport;
use crate::source_graph::SourceContextId;

#[derive(Debug)]
pub(crate) struct LegacyFrontEnd {
    resolver: resolver::LegacyResolver,
    lowering: lowering::LegacyLoweringPlan,
}

impl LegacyFrontEnd {
    pub(crate) fn new(dialect: PineDialect) -> Self {
        debug_assert!(catalog::validate_catalog(LEGACY_RULES).is_empty());
        Self {
            resolver: resolver::LegacyResolver::new(dialect),
            lowering: lowering::LegacyLoweringPlan::new(),
        }
    }

    pub(crate) fn with_rules(dialect: PineDialect, rules: &'static [LegacyRule]) -> Self {
        Self {
            resolver: resolver::LegacyResolver::with_rules(dialect, rules),
            lowering: lowering::LegacyLoweringPlan::new(),
        }
    }

    pub(crate) fn resolve_call(&self, source_name: &str) -> Option<LegacyResolution> {
        self.resolver.resolve_call(source_name)
    }

    pub(crate) fn resolve_value(&self, source_name: &str) -> Option<LegacyResolution> {
        self.resolver.resolve_value(source_name)
    }

    pub(crate) fn record_call_translation(
        &mut self,
        report: &mut CompatibilityReport,
        source_context_id: SourceContextId,
        span: Span,
        rule: LegacyRule,
    ) {
        let canonical_name = rule
            .canonical_name
            .expect("validated exact legacy rule has a canonical target");
        self.lowering
            .record_call(source_context_id, span, canonical_name);
        report::record_exact_translation(report, rule, span);
    }

    pub(crate) fn record_value_translation(
        &mut self,
        report: &mut CompatibilityReport,
        source_context_id: SourceContextId,
        span: Span,
        rule: LegacyRule,
    ) {
        let canonical_name = rule
            .canonical_name
            .expect("validated exact legacy rule has a canonical target");
        self.lowering
            .record_value(source_context_id, span, canonical_name);
        report::record_exact_translation(report, rule, span);
    }

    pub(crate) fn canonical_call_name(
        &self,
        source_context_id: SourceContextId,
        span: Span,
    ) -> Option<&'static str> {
        self.lowering.call_name(source_context_id, span)
    }

    pub(crate) fn canonical_value_name(
        &self,
        source_context_id: SourceContextId,
        span: Span,
    ) -> Option<&'static str> {
        self.lowering.value_name(source_context_id, span)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SourcePolicy {
    pub(crate) language: LanguageSelection,
    pub(crate) script_mode: ScriptModeClassification,
    pub(crate) legacy_admission_failure: Option<LegacyAdmissionFailure>,
}

impl SourcePolicy {
    pub(crate) fn from_program_with_implicit(
        program: &Program,
        implicit_dialect: PineDialect,
    ) -> Self {
        let language = LanguageSelection::from_program_with_implicit(program, implicit_dialect);
        let script_mode = declarations::classify_script_mode(program);
        let legacy_admission_failure = language
            .dialect
            .and_then(|dialect| legacy_admission_failure(program, dialect));
        Self {
            language,
            script_mode,
            legacy_admission_failure,
        }
    }
}
