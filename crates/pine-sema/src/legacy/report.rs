use pine_syntax::Span;

use crate::compatibility::{CompatibilityReport, LegacyTranslation, LegacyTranslationKind};

use super::catalog::{LegacyRule, LegacyRuleKind};

pub(crate) fn record_exact_translation(
    report: &mut CompatibilityReport,
    rule: LegacyRule,
    span: Span,
) {
    let canonical_feature = rule
        .canonical_name
        .expect("validated exact legacy rules have canonical targets");
    let kind = match rule.kind {
        LegacyRuleKind::ExactFunctionAlias => LegacyTranslationKind::ExactAlias,
        LegacyRuleKind::ExactSymbolAlias => LegacyTranslationKind::SymbolAlias,
        _ => panic!("focused legacy rules cannot use exact translation reporting"),
    };
    report.legacy_translations.push(LegacyTranslation {
        source_feature: rule.source_name.to_owned(),
        canonical_feature: canonical_feature.to_owned(),
        kind,
        span,
    });
}

pub(crate) fn record_signature_translation(
    report: &mut CompatibilityReport,
    rule: LegacyRule,
    span: Span,
) {
    let canonical_feature = rule
        .canonical_name
        .expect("validated focused declaration has a canonical target");
    report.legacy_translations.push(LegacyTranslation {
        source_feature: rule.source_name.to_owned(),
        canonical_feature: canonical_feature.to_owned(),
        kind: LegacyTranslationKind::SignatureReshape,
        span,
    });
}

pub(crate) fn record_constant_translation(
    report: &mut CompatibilityReport,
    rule: LegacyRule,
    span: Span,
) {
    let canonical_feature = rule
        .canonical_name
        .expect("validated legacy input constant has a canonical target");
    report.legacy_translations.push(LegacyTranslation {
        source_feature: rule.source_name.to_owned(),
        canonical_feature: canonical_feature.to_owned(),
        kind: LegacyTranslationKind::ConstantAlias,
        span,
    });
}

pub(crate) fn record_input_signature_translation(
    report: &mut CompatibilityReport,
    rule: LegacyRule,
    canonical_feature: &'static str,
    span: Span,
) {
    report.legacy_translations.push(LegacyTranslation {
        source_feature: rule.source_name.to_owned(),
        canonical_feature: canonical_feature.to_owned(),
        kind: LegacyTranslationKind::SignatureReshape,
        span,
    });
}

pub(crate) fn normalize_legacy_report(report: &mut CompatibilityReport) {
    report.legacy_translations.sort_by(|left, right| {
        (
            left.span.start,
            left.span.end,
            left.source_feature.as_str(),
            left.canonical_feature.as_str(),
            left.kind.name(),
        )
            .cmp(&(
                right.span.start,
                right.span.end,
                right.source_feature.as_str(),
                right.canonical_feature.as_str(),
                right.kind.name(),
            ))
    });
    report.legacy_translations.dedup();

    report.legacy_emulations.sort_by(|left, right| {
        (
            left.span.start,
            left.span.end,
            left.feature.as_str(),
            left.behavior.as_str(),
        )
            .cmp(&(
                right.span.start,
                right.span.end,
                right.feature.as_str(),
                right.behavior.as_str(),
            ))
    });
    report.legacy_emulations.dedup();
}
