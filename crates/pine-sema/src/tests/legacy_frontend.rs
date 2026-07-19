use pine_ir::{HirExpr, HirExprKind, HirStmtKind};
use pine_syntax::{SourceFile, Span};

use crate::LegacyTranslationKind;
use crate::compatibility::{CompatibilityReport, LegacyEmulation, LegacyTranslation};
use crate::legacy::{
    CatalogValidationError, LegacyRule, LegacyRuleKind, LegacyRuleSupport, PineDialect,
    validate_catalog,
};

const TEST_RULES: &[LegacyRule] = &[
    LegacyRule {
        source_name: "iff",
        canonical_name: None,
        min_version: PineDialect::V1,
        max_version: PineDialect::V4,
        kind: LegacyRuleKind::FocusedExpression,
        support: LegacyRuleSupport::UnsupportedKnown {
            reason: "test deferred expression",
        },
    },
    LegacyRule {
        source_name: "sma",
        canonical_name: Some("ta.sma"),
        min_version: PineDialect::V1,
        max_version: PineDialect::V4,
        kind: LegacyRuleKind::ExactFunctionAlias,
        support: LegacyRuleSupport::Supported,
    },
    LegacyRule {
        source_name: "tickerid",
        canonical_name: Some("syminfo.tickerid"),
        min_version: PineDialect::V1,
        max_version: PineDialect::V3,
        kind: LegacyRuleKind::ExactSymbolAlias,
        support: LegacyRuleSupport::Supported,
    },
];

fn analyze_legacy(source: &str) -> crate::Analysis {
    crate::analysis::analyze_source_with_legacy_rules(
        &SourceFile::new("legacy-test.pine", source),
        TEST_RULES,
    )
}

fn plot_arg(analysis: &crate::Analysis) -> &HirExpr {
    analysis
        .hir
        .as_ref()
        .expect("HIR")
        .statements
        .iter()
        .find_map(|statement| match &statement.kind {
            HirStmtKind::Expr(HirExpr {
                kind: HirExprKind::Call { callee, args, .. },
                ..
            }) if callee == "plot" => args.first().map(|arg| &arg.value),
            _ => None,
        })
        .expect("plot argument")
}

fn diagnostic_codes(analysis: &crate::Analysis) -> Vec<&str> {
    analysis
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

#[test]
fn exact_function_alias_lowers_to_canonical_hir_and_preserves_source_span() {
    let source = "//@version=4\nplot(sma(close, 2))\n";
    let analysis = analyze_legacy(source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let plot_arg = plot_arg(&analysis);
    assert!(matches!(
        &plot_arg.kind,
        HirExprKind::Call { callee, .. } if callee == "ta.sma"
    ));
    let translation = analysis
        .compatibility
        .legacy_translations
        .first()
        .expect("translation");
    let start = source.find("sma").expect("source alias");
    assert_eq!(translation.source_feature, "sma");
    assert_eq!(translation.canonical_feature, "ta.sma");
    assert_eq!(translation.kind, LegacyTranslationKind::ExactAlias);
    assert_eq!(translation.span, Span::new(start, start + "sma".len()));
}

#[test]
fn exact_alias_applies_across_only_its_declared_version_range() {
    for directive in [None, Some(2), Some(3), Some(4)] {
        let source = match directive {
            Some(version) => format!("//@version={version}\nplot(sma(close, 2))\n"),
            None => "plot(sma(close, 2))\n".to_owned(),
        };
        let analysis = analyze_legacy(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        assert_eq!(analysis.compatibility.legacy_translations.len(), 1);
    }

    let v4_symbol = analyze_legacy("//@version=4\nplot(str.length(tickerid))\n");
    assert_eq!(diagnostic_codes(&v4_symbol), vec!["E_UNKNOWN_SYMBOL"]);
    assert!(v4_symbol.compatibility.legacy_translations.is_empty());
}

#[test]
fn user_defined_function_wins_over_exact_alias() {
    let analysis =
        analyze_legacy("//@version=4\nsma(source, length) => source\nplot(sma(close, 2))\n");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.legacy_translations.is_empty());
    assert!(!matches!(
        &plot_arg(&analysis).kind,
        HirExprKind::Call { callee, .. } if callee == "ta.sma"
    ));
}

#[test]
fn lexical_value_prevents_function_alias_fallback() {
    let analysis = analyze_legacy("//@version=4\nsma = close\nplot(sma(close, 2))\n");
    assert_eq!(diagnostic_codes(&analysis), vec!["E_UNKNOWN_FUNCTION"]);
    assert!(analysis.compatibility.legacy_translations.is_empty());
}

#[test]
fn exact_symbol_alias_is_fallback_and_lowers_to_canonical_builtin() {
    let source = "//@version=3\nplot(str.length(tickerid))\n";
    let analysis = analyze_legacy(source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let HirExprKind::Call { callee, args, .. } = &plot_arg(&analysis).kind else {
        panic!("expected str.length call")
    };
    assert_eq!(callee, "str.length");
    assert!(matches!(
        args.first().map(|arg| &arg.value.kind),
        Some(HirExprKind::Builtin(name)) if name == "syminfo.tickerid"
    ));
    let translation = analysis
        .compatibility
        .legacy_translations
        .first()
        .expect("translation");
    assert_eq!(translation.kind, LegacyTranslationKind::SymbolAlias);
    assert_eq!(translation.span.start, source.find("tickerid").unwrap());
}

#[test]
fn lexical_symbol_wins_over_symbol_alias() {
    let analysis =
        analyze_legacy("//@version=3\ntickerid = \"local\"\nplot(str.length(tickerid))\n");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.legacy_translations.is_empty());
}

#[test]
fn unsupported_known_and_unknown_calls_remain_distinct() {
    let unsupported = analyze_legacy("//@version=4\nplot(iff(true, close, open))\n");
    assert_eq!(
        diagnostic_codes(&unsupported),
        vec!["E_UNSUPPORTED_FEATURE"]
    );
    assert_eq!(unsupported.compatibility.unsupported[0].feature, "iff");

    let unknown = analyze_legacy("//@version=4\nplot(mystery(close))\n");
    assert_eq!(diagnostic_codes(&unknown), vec!["E_UNKNOWN_FUNCTION"]);
    assert!(unknown.compatibility.unsupported.is_empty());
}

#[test]
fn modern_dialects_do_not_activate_legacy_aliases() {
    for version in [5, 6] {
        let source = format!("//@version={version}\nindicator(\"modern\")\nplot(sma(close, 2))\n");
        let analysis = crate::analysis::analyze_source_with_legacy_rules(
            &SourceFile::new("modern-control.pine", source),
            TEST_RULES,
        );
        assert_eq!(diagnostic_codes(&analysis), vec!["E_UNKNOWN_FUNCTION"]);
        assert!(analysis.compatibility.legacy_translations.is_empty());
    }
}

#[test]
fn canonical_name_in_legacy_source_is_not_reported_as_translation() {
    let analysis = analyze_legacy("//@version=4\nplot(ta.sma(close, 2))\n");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.legacy_translations.is_empty());
    assert!(matches!(
        &plot_arg(&analysis).kind,
        HirExprKind::Call { callee, .. } if callee == "ta.sma"
    ));
}

#[test]
fn production_and_synthetic_catalogs_validate_against_canonical_registries() {
    assert!(validate_catalog(crate::legacy::LEGACY_RULES).is_empty());
    assert!(validate_catalog(TEST_RULES).is_empty());

    const INVALID: &[LegacyRule] = &[LegacyRule {
        source_name: "bad",
        canonical_name: Some("ta.does_not_exist"),
        min_version: PineDialect::V1,
        max_version: PineDialect::V4,
        kind: LegacyRuleKind::ExactFunctionAlias,
        support: LegacyRuleSupport::Supported,
    }];
    let errors = validate_catalog(INVALID);
    assert!(
        matches!(errors.as_slice(), [CatalogValidationError(message)] if message.contains("not registered"))
    );

    const BAD_SYMBOL: &[LegacyRule] = &[LegacyRule {
        source_name: "bad_symbol",
        canonical_name: Some("syminfo.does_not_exist"),
        min_version: PineDialect::V1,
        max_version: PineDialect::V3,
        kind: LegacyRuleKind::ExactSymbolAlias,
        support: LegacyRuleSupport::Supported,
    }];
    assert!(
        validate_catalog(BAD_SYMBOL)
            .iter()
            .any(|error| error.0.contains("canonical symbol"))
    );

    const MODERN_LEAK: &[LegacyRule] = &[LegacyRule {
        source_name: "leak",
        canonical_name: Some("ta.sma"),
        min_version: PineDialect::V4,
        max_version: PineDialect::V5,
        kind: LegacyRuleKind::ExactFunctionAlias,
        support: LegacyRuleSupport::Supported,
    }];
    assert!(
        validate_catalog(MODERN_LEAK)
            .iter()
            .any(|error| error.0.contains("modern dialects"))
    );

    const OVERLAP: &[LegacyRule] = &[
        LegacyRule {
            source_name: "same",
            canonical_name: Some("ta.sma"),
            min_version: PineDialect::V1,
            max_version: PineDialect::V3,
            kind: LegacyRuleKind::ExactFunctionAlias,
            support: LegacyRuleSupport::Supported,
        },
        LegacyRule {
            source_name: "same",
            canonical_name: Some("ta.ema"),
            min_version: PineDialect::V3,
            max_version: PineDialect::V4,
            kind: LegacyRuleKind::ExactFunctionAlias,
            support: LegacyRuleSupport::Supported,
        },
    ];
    assert!(
        validate_catalog(OVERLAP)
            .iter()
            .any(|error| error.0.contains("overlapping"))
    );
}

#[test]
fn legacy_report_normalization_is_stable_and_deduplicated() {
    let first = LegacyTranslation {
        source_feature: "sma".to_owned(),
        canonical_feature: "ta.sma".to_owned(),
        kind: LegacyTranslationKind::ExactAlias,
        span: Span::new(20, 23),
    };
    let earlier = LegacyTranslation {
        source_feature: "tickerid".to_owned(),
        canonical_feature: "syminfo.tickerid".to_owned(),
        kind: LegacyTranslationKind::SymbolAlias,
        span: Span::new(10, 18),
    };
    let emulation = LegacyEmulation {
        feature: "session".to_owned(),
        behavior: "legacy default".to_owned(),
        span: Span::new(30, 37),
    };
    let mut report = CompatibilityReport {
        legacy_translations: vec![first.clone(), earlier.clone(), first],
        legacy_emulations: vec![emulation.clone(), emulation],
        ..CompatibilityReport::default()
    };
    crate::legacy::normalize_legacy_report(&mut report);
    assert_eq!(report.legacy_translations.len(), 2);
    assert_eq!(report.legacy_translations[0], earlier);
    assert_eq!(report.legacy_translations[1].source_feature, "sma");
    assert_eq!(report.legacy_emulations.len(), 1);
}
