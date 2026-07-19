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

fn analyze_production(source: &str) -> crate::Analysis {
    crate::analyze_source(&SourceFile::new("legacy-production.pine", source))
}

fn normalized_hir(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_production(source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let mut hir = analysis.hir.expect("HIR");
    hir.language_version = None;
    hir
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

fn top_level_call<'a>(analysis: &'a crate::Analysis, name: &str) -> &'a HirExpr {
    analysis
        .hir
        .as_ref()
        .expect("HIR")
        .statements
        .iter()
        .find_map(|statement| match &statement.kind {
            HirStmtKind::Expr(
                expr @ HirExpr {
                    kind: HirExprKind::Call { callee, .. },
                    ..
                },
            ) if callee == name => Some(expr),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing top-level `{name}` call"))
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
fn production_v4_study_and_first_alias_batch_match_canonical_hir() {
    let legacy = include_str!("../../../../tests/fixtures/legacy/v4/runtime/aliases_legacy.pine");
    let canonical =
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/aliases_canonical.pine");

    assert_eq!(normalized_hir(legacy), normalized_hir(canonical));

    let analysis = analyze_production(legacy);
    let translations = analysis
        .compatibility
        .legacy_translations
        .iter()
        .map(|translation| {
            (
                translation.source_feature.as_str(),
                translation.canonical_feature.as_str(),
                translation.kind,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        translations,
        vec![
            (
                "study",
                "indicator",
                LegacyTranslationKind::SignatureReshape
            ),
            ("bb", "ta.bb", LegacyTranslationKind::ExactAlias),
            ("ema", "ta.ema", LegacyTranslationKind::ExactAlias),
            ("sma", "ta.sma", LegacyTranslationKind::ExactAlias),
            (
                "crossover",
                "ta.crossover",
                LegacyTranslationKind::ExactAlias
            ),
            ("abs", "math.abs", LegacyTranslationKind::ExactAlias),
        ]
    );
}

#[test]
fn v4_study_binds_historical_positions_before_canonical_validation() {
    let legacy = include_str!("../../../../tests/fixtures/legacy/v4/sema/declaration_legacy.pine");
    let canonical =
        include_str!("../../../../tests/fixtures/legacy/v4/sema/declaration_canonical.pine");

    assert_eq!(normalized_hir(legacy), normalized_hir(canonical));

    let analysis = analyze_production(legacy);
    let hir = analysis.hir.expect("legacy declaration HIR");
    assert_eq!(hir.max_bars_back, Some(12));
    assert_eq!(hir.drawing_settings.max_lines_count, Some(75));
    assert_eq!(hir.drawing_settings.max_labels_count, Some(80));
    assert_eq!(hir.drawing_settings.max_boxes_count, Some(65));
}

#[test]
fn production_v4_input_overloads_match_canonical_hir() {
    let legacy = include_str!("../../../../tests/fixtures/legacy/v4/runtime/inputs_legacy.pine");
    let canonical =
        include_str!("../../../../tests/fixtures/legacy/v4/runtime/inputs_canonical.pine");

    assert_eq!(normalized_hir(legacy), normalized_hir(canonical));

    let analysis = analyze_production(legacy);
    let input_targets = analysis
        .compatibility
        .legacy_translations
        .iter()
        .filter(|translation| translation.source_feature == "input")
        .map(|translation| translation.canonical_feature.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        input_targets,
        vec![
            "input.int",
            "input.float",
            "input.bool",
            "input.color",
            "input.string",
            "input.symbol",
            "input.timeframe",
            "input.session",
            "input.source",
            "input.time",
            "input.price",
        ]
    );
    assert!(
        analysis
            .compatibility
            .legacy_translations
            .iter()
            .filter(|translation| translation.source_feature.starts_with("input."))
            .all(|translation| translation.kind == LegacyTranslationKind::ConstantAlias)
    );
}

#[test]
fn v4_input_type_constant_survives_a_local_const_alias() {
    let source =
        include_str!("../../../../tests/fixtures/legacy/v4/sema/input_constant_alias.pine");
    let analysis = analyze_production(source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert_eq!(
        analysis
            .compatibility
            .legacy_translations
            .iter()
            .filter(|translation| translation.source_feature == "input.integer")
            .count(),
        1
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn v4_input_infers_fixture_backed_scalar_and_source_overloads() {
    let analysis = analyze_production(
        "//@version=4\nstudy(\"inferred\")\ni = input(1)\nf = input(1.5)\nb = input(true)\ns = input(\"text\")\nc = input(color.red)\nsrc = input(close)\nplot(src)\n",
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert_eq!(
        analysis
            .compatibility
            .legacy_translations
            .iter()
            .filter(|translation| translation.source_feature == "input")
            .map(|translation| translation.canonical_feature.as_str())
            .collect::<Vec<_>>(),
        vec![
            "input.int",
            "input.float",
            "input.bool",
            "input.string",
            "input.color",
            "input.source",
        ]
    );
}

#[test]
fn v4_input_rejects_ambiguous_and_wrong_overloads_before_runtime() {
    let string_type = analyze_production(
        "//@version=4\nstudy(\"bad\")\nlength = input(3, \"Length\", type=\"input.int\")\nplot(close)\n",
    );
    assert_eq!(
        diagnostic_codes(&string_type),
        vec!["E_LEGACY_INPUT_OVERLOAD"]
    );
    assert!(string_type.hir.is_none());

    let source_confirm = analyze_production(
        "//@version=4\nstudy(\"bad\")\nsrc = input(close, \"Source\", type=input.source, confirm=true)\nplot(close)\n",
    );
    assert_eq!(diagnostic_codes(&source_confirm), vec!["E_CALL_ARG_NAME"]);
    assert!(source_confirm.hir.is_none());

    let no_defval = analyze_production(
        "//@version=4\nstudy(\"bad\")\nlength = input(title=\"Length\", type=input.integer)\nplot(close)\n",
    );
    assert_eq!(diagnostic_codes(&no_defval), vec!["E_CALL_ARITY"]);
    assert!(no_defval.hir.is_none());
}

#[test]
fn v4_integer_input_accepts_float_metadata_without_widening_modern_input_int() {
    let legacy = analyze_production(
        "//@version=4\nstudy(\"bounds\")\nlength = input(1, \"Length\", input.integer, minval=1.0, maxval=5.0, step=1.0)\nplot(sma(close, length))\n",
    );
    assert!(legacy.diagnostics.is_empty(), "{:?}", legacy.diagnostics);
    assert!(legacy.hir.is_some());

    let modern = analyze_production(
        "//@version=5\nindicator(\"bounds\")\nlength = input.int(1, \"Length\", minval=1.0)\nplot(ta.sma(close, length))\n",
    );
    assert_eq!(diagnostic_codes(&modern), vec!["E_CALL_ARG_TYPE"]);
    assert!(modern.compatibility.legacy_translations.is_empty());
    assert!(modern.hir.is_none());
}

#[test]
fn v4_input_constants_do_not_become_global_numeric_coercions() {
    let analysis = analyze_production("//@version=4\nstudy(\"bad\")\nplot(input.integer * 2)\n");
    assert!(analysis.hir.is_none());
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("numeric")),
        "{:?}",
        analysis.diagnostics
    );
}

#[test]
fn modern_sources_reject_v4_input_type_constants() {
    for version in [5, 6] {
        let analysis = analyze_production(&format!(
            "//@version={version}\nindicator(\"modern\")\nkind = input.integer\nplot(close)\n"
        ));
        assert!(analysis.hir.is_none());
        assert!(analysis.compatibility.legacy_translations.is_empty());
    }
}

#[test]
fn v4_study_timeframe_arguments_produce_one_focused_failure() {
    let analysis = analyze_production(
        "//@version=4\nstudy(\"MTF\", resolution=\"D\", resolution_gaps=true)\nplot(close)\n",
    );

    assert_eq!(diagnostic_codes(&analysis), vec!["E_UNSUPPORTED_FEATURE"]);
    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "study.resolution"
    );
    assert!(analysis.compatibility.legacy_translations.is_empty());
    assert!(analysis.hir.is_none());
}

#[test]
fn v4_study_rejects_unmapped_declaration_options_before_lowering() {
    let analysis = analyze_production(
        "//@version=4\nstudy(\"z-order\", explicit_plot_zorder=true)\nplot(close)\n",
    );

    assert_eq!(diagnostic_codes(&analysis), vec!["E_UNSUPPORTED_FEATURE"]);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "study.explicit_plot_zorder"
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn v4_session_call_is_guarded_until_legacy_default_semantics_land() {
    let analysis =
        analyze_production("//@version=4\nstudy(\"session\")\nplot(time(\"D\", \"0930-1600\"))\n");

    assert_eq!(diagnostic_codes(&analysis), vec!["E_UNSUPPORTED_FEATURE"]);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "time.session"
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn production_aliases_still_yield_to_v4_user_functions_and_lexical_values() {
    let udf = analyze_production(
        "//@version=4\nstudy(\"collision\")\nsma(source, length) => source\nplot(sma(close, 2))\n",
    );
    assert!(udf.diagnostics.is_empty(), "{:?}", udf.diagnostics);
    assert_eq!(
        udf.compatibility
            .legacy_translations
            .iter()
            .map(|translation| translation.source_feature.as_str())
            .collect::<Vec<_>>(),
        vec!["study"]
    );

    let lexical = analyze_production(
        "//@version=4\nstudy(\"collision\")\nsma = close\nplot(sma(close, 2))\n",
    );
    assert_eq!(diagnostic_codes(&lexical), vec!["E_UNKNOWN_FUNCTION"]);
    assert!(
        lexical
            .compatibility
            .legacy_translations
            .iter()
            .all(|translation| translation.source_feature != "sma")
    );
}

#[test]
fn modern_sources_reject_every_production_v4_alias() {
    for version in [5, 6] {
        for alias_call in [
            "sma(close, 2)",
            "ema(close, 2)",
            "bb(close, 2, 2)",
            "crossover(close, open)",
            "abs(close)",
        ] {
            let analysis = analyze_production(&format!(
                "//@version={version}\nindicator(\"modern\")\nplot({alias_call})\n"
            ));
            assert_eq!(diagnostic_codes(&analysis), vec!["E_UNKNOWN_FUNCTION"]);
            assert!(analysis.compatibility.legacy_translations.is_empty());
        }
    }
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

#[test]
fn v4_output_binder_accepts_historical_signatures_and_retains_hidden_semantics() {
    let source = r#"//@version=4
study("v4 outputs")
selectedStyle = input(1)
p1 = plot(close, "p1", color.blue, 2, selectedStyle, true, 40, 0, 1, true, true, 3, display.all)
p2 = plot(open)
plotchar(close > open, "char", "X", location.abovebar, color.red, 25)
plotshape(close > open, "shape", shape.circle, location.belowbar, color.green, 10)
plotarrow(close - open, "arrow", color.green, color.red, 35)
plotbar(open, high, low, close, "bars", color.blue)
plotcandle(open, high, low, close, "candles", color.blue, color.gray)
h1 = hline(1, "one", color.gray, 1)
h2 = hline(2)
fill(p1, p2, color.blue, 55)
fill(hline1=h1, hline2=h2, color=color.red)
bgcolor(color.blue, 45, title="bg")
barcolor(color.red, title="bars")
"#;
    let analysis = analyze_production(source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let plot = top_level_call(&analysis, "plotchar");
    let HirExprKind::Call { args, .. } = &plot.kind else {
        unreachable!()
    };
    assert!(
        args.iter()
            .any(|arg| { arg.name.as_deref() == Some(pine_ir::LEGACY_TRANSPARENCY_ARG) })
    );

    let translations = analysis
        .compatibility
        .legacy_translations
        .iter()
        .filter(|translation| translation.kind == LegacyTranslationKind::OutputAdaptation)
        .collect::<Vec<_>>();
    assert_eq!(translations.len(), 8);
    assert!(
        analysis
            .compatibility
            .legacy_emulations
            .iter()
            .any(|item| { item.feature == "plot.transp" })
    );
    assert!(
        analysis
            .compatibility
            .legacy_emulations
            .iter()
            .any(|item| { item.feature == "plot.numeric_style" })
    );
    assert!(
        analysis
            .compatibility
            .legacy_emulations
            .iter()
            .any(|item| { item.feature == "hline.numeric_style" })
    );
}

#[test]
fn v4_output_binder_rejects_unsupported_arguments_and_invalid_legacy_values() {
    for (call, expected_code) in [
        ("plot(close, force_overlay=true)", "E_CALL_ARG_NAME"),
        ("plot(close, transp=close)", "E_LEGACY_OUTPUT_ARGUMENT"),
        ("plot(close, style=9)", "E_LEGACY_OUTPUT_ARGUMENT"),
        (
            "plot(close, style=\"invented\")",
            "E_LEGACY_OUTPUT_ARGUMENT",
        ),
        ("hline(1, linestyle=3)", "E_LEGACY_OUTPUT_ARGUMENT"),
    ] {
        let source = format!("//@version=4\nstudy(\"invalid\")\n{call}\n");
        let analysis = analyze_production(&source);
        assert!(
            diagnostic_codes(&analysis).contains(&expected_code),
            "{call}: {:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none(), "{call}");
    }

    let mixed_fill = analyze_production(
        "//@version=4\nstudy(\"invalid fill\")\np = plot(close)\nh = hline(1)\nfill(p, h)\n",
    );
    assert_eq!(
        diagnostic_codes(&mixed_fill),
        vec!["E_LEGACY_OUTPUT_ARGUMENT"]
    );
}

#[test]
fn legacy_output_compatibility_does_not_weaken_modern_unique_types() {
    for version in [5, 6] {
        for call in [
            "plot(close, transp=40)",
            "plot(close, style=1)",
            "hline(1, linestyle=1)",
        ] {
            let source = format!("//@version={version}\nindicator(\"modern\")\n{call}\n");
            let analysis = analyze_production(&source);
            assert!(!analysis.diagnostics.is_empty(), "{version}: {call}");
            assert!(analysis.compatibility.legacy_translations.is_empty());
            assert!(analysis.compatibility.legacy_emulations.is_empty());
        }
    }
}
