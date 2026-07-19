use pine_ir::{
    HirExpr, HirExprKind, HirHistoryOffset, HirStmtKind, PineType, Qualifier, ValueKind,
};
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
        source_name: "security",
        canonical_name: Some("request.security"),
        min_version: PineDialect::V1,
        max_version: PineDialect::V4,
        kind: LegacyRuleKind::FocusedSecurity,
        support: LegacyRuleSupport::Supported,
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

fn analyze_catalog_without_admission(source: &str) -> crate::Analysis {
    crate::analysis::analyze_source_with_legacy_rules(
        &SourceFile::new("legacy-catalog-test.pine", source),
        crate::legacy::LEGACY_RULES,
    )
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

fn plot_call_name(analysis: &crate::Analysis) -> &str {
    let HirExprKind::Call { callee, .. } = &plot_arg(analysis).kind else {
        panic!("plot argument is not a call");
    };
    callee
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
fn legacy_security_defaults_and_explicit_merge_values_are_versioned() {
    let v2 = analyze_legacy("//@version=2\nplot(security(\"NYSE:IBM\", \"5\", close, false))\n");
    assert!(v2.diagnostics.is_empty(), "{:?}", v2.diagnostics);
    assert_eq!(
        plot_call_name(&v2),
        "$legacy.security.gaps_off.lookahead_on"
    );

    let v3 = analyze_legacy(
        "//@version=3\nplot(security(symbol=\"NYSE:IBM\", resolution=\"5\", expression=close))\n",
    );
    assert!(v3.diagnostics.is_empty(), "{:?}", v3.diagnostics);
    assert_eq!(
        plot_call_name(&v3),
        "$legacy.security.gaps_off.lookahead_off"
    );

    let v3_explicit = analyze_legacy(
        "//@version=3\nplot(security(\"NYSE:IBM\", \"5\", close, gaps=barmerge.gaps_on, lookahead=true))\n",
    );
    assert!(
        v3_explicit.diagnostics.is_empty(),
        "{:?}",
        v3_explicit.diagnostics
    );
    assert_eq!(
        plot_call_name(&v3_explicit),
        "$legacy.security.gaps_on.lookahead_on"
    );
}

#[test]
fn production_v4_security_uses_request_expression_analysis_and_source_spanned_hir() {
    let source = "//@version=4\nstudy(\"legacy security\")\nplot(security(syminfo.tickerid, \"5\", sma(close, 2)))\n";
    let analysis = analyze_production(source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let HirExprKind::Call { callee, args, .. } = &plot_arg(&analysis).kind else {
        panic!("legacy security was not lowered to a call");
    };
    assert_eq!(callee, "$legacy.security.gaps_off.lookahead_off");
    assert_eq!(args.len(), 5);
    assert_eq!(args[3].name.as_deref(), Some("$legacy_span_start"));
    assert_eq!(args[4].name.as_deref(), Some("$legacy_span_end"));
    let HirExprKind::Call { callee, .. } = &args[2].value.kind else {
        panic!("requested SMA expression was not lowered");
    };
    assert_eq!(callee, "ta.sma");
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .legacy_translations
            .iter()
            .any(|item| {
                item.source_feature == "security" && item.canonical_feature == "request.security"
            })
    );
}

#[test]
fn legacy_security_rejects_invalid_versioned_signatures_and_dynamic_merge_metadata() {
    let named_v2 = analyze_legacy(
        "//@version=2\nplot(security(symbol=\"NYSE:IBM\", resolution=\"5\", expression=close))\n",
    );
    assert!(diagnostic_codes(&named_v2).contains(&"E_CALL_ARG_NAME"));

    let dynamic_merge = analyze_legacy(
        "//@version=4\nflag = close > open\nplot(security(\"NYSE:IBM\", \"5\", close, flag))\n",
    );
    assert!(diagnostic_codes(&dynamic_merge).contains(&"E_LEGACY_SECURITY_MERGE"));

    let sixth_arg = analyze_legacy(
        "//@version=4\nplot(security(\"NYSE:IBM\", \"5\", close, false, false, false))\n",
    );
    assert!(diagnostic_codes(&sixth_arg).contains(&"E_CALL_ARITY"));
}

#[test]
fn user_defined_security_function_shadows_legacy_builtin() {
    let analysis = analyze_production(
        "//@version=4\nstudy(\"shadow\")\nsecurity(symbol, resolution, expression) => expression + 1\nplot(security(\"X\", \"5\", close))\n",
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(!matches!(
        &plot_arg(&analysis).kind,
        HirExprKind::Call { callee, .. }
            if callee == "$legacy.security.gaps_off.lookahead_off"
    ));
    assert!(
        analysis
            .compatibility
            .legacy_translations
            .iter()
            .all(|item| item.source_feature != "security")
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
fn v4_session_calls_lower_after_versioned_default_semantics_land() {
    let analysis = analyze_production(
        "//@version=4\nstudy(\"session\")\nplot(time_close(\"D\", \"0930-1600\"))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(matches!(
        &plot_arg(&analysis).kind,
        HirExprKind::Call { callee, .. } if callee == "time_close"
    ));
}

#[test]
fn v4_iff_lowers_to_strict_internal_select_with_source_spanned_report() {
    let source = "//@version=4\nstudy(\"iff\")\nplot(iff(close > open, high, low))\n";
    let analysis = analyze_production(source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let HirExprKind::Call { callee, args, .. } = &plot_arg(&analysis).kind else {
        panic!("expected strict iff call")
    };
    assert_eq!(callee, "$legacy.iff");
    assert_eq!(
        args.iter()
            .map(|arg| arg.name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("condition"), Some("result1"), Some("result2")]
    );
    let translation = analysis
        .compatibility
        .legacy_translations
        .iter()
        .find(|translation| translation.source_feature == "iff")
        .expect("iff translation");
    assert_eq!(translation.kind, LegacyTranslationKind::ExpressionDesugar);
    assert_eq!(translation.span.start, source.find("iff(close").unwrap());
    assert!(
        analysis
            .compatibility
            .legacy_emulations
            .iter()
            .any(|emulation| emulation.feature == "iff")
    );
}

#[test]
fn v4_offset_lowers_to_native_history_and_retention_requirement() {
    let analysis = analyze_production(
        "//@version=4\nstudy(\"offset\")\nplot(offset(source=close, offset=2))\n",
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(matches!(
        &plot_arg(&analysis).kind,
        HirExprKind::History {
            offset: HirHistoryOffset::Constant(2),
            ..
        }
    ));
    let hir = analysis.hir.expect("HIR");
    assert_eq!(hir.history.max_constant_offset, 2);
    assert!(
        hir.series_history
            .iter()
            .any(|requirement| requirement.max_constant_offset == 2)
    );
}

#[test]
fn v4_rsi_overload_selection_is_type_directed() {
    let length = analyze_production(
        "//@version=4\nstudy(\"length\")\nlength=input(2)\nplot(rsi(close, length))\n",
    );
    assert!(length.diagnostics.is_empty(), "{:?}", length.diagnostics);
    assert!(matches!(
        &plot_arg(&length).kind,
        HirExprKind::Call { callee, .. } if callee == "ta.rsi"
    ));

    for second_argument in ["2.0", "bar_index"] {
        let series = analyze_production(&format!(
            "//@version=4\nstudy(\"series overload\")\nplot(rsi(close, {second_argument}))\n"
        ));
        assert!(
            series.diagnostics.is_empty(),
            "{second_argument}: {:?}",
            series.diagnostics
        );
        assert!(
            matches!(
                &plot_arg(&series).kind,
                HirExprKind::Call { callee, .. } if callee == "$legacy.rsi_series"
            ),
            "{second_argument}"
        );
    }

    let series =
        analyze_production("//@version=4\nstudy(\"series\")\nplot(rsi(x=close, y=open))\n");
    assert!(series.diagnostics.is_empty(), "{:?}", series.diagnostics);
    assert!(matches!(
        &plot_arg(&series).kind,
        HirExprKind::Call { callee, .. } if callee == "$legacy.rsi_series"
    ));
    assert!(
        series
            .compatibility
            .legacy_emulations
            .iter()
            .any(|emulation| emulation.feature == "rsi")
    );

    let invalid =
        analyze_production("//@version=4\nstudy(\"invalid\")\nplot(rsi(close, close > open))\n");
    assert_eq!(diagnostic_codes(&invalid), vec!["E_LEGACY_RSI_OVERLOAD"]);
    assert!(invalid.hir.is_none());
}

#[test]
fn focused_legacy_calls_yield_to_user_defined_functions() {
    let analysis = analyze_production(
        "//@version=4\nstudy(\"shadow\")\niff(condition, result1, result2) => result1\nplot(iff(true, close, open))\n",
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .legacy_translations
            .iter()
            .all(|translation| translation.source_feature != "iff")
    );
    assert!(!matches!(
        &plot_arg(&analysis).kind,
        HirExprKind::Call { callee, .. } if callee == "$legacy.iff"
    ));
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
            "change(close)",
            "crossover(close, open)",
            "highest(high, 2)",
            "lowest(low, 2)",
            "max(close, open)",
            "min(close, open)",
            "abs(close)",
            "iff(true, close, open)",
            "offset(close, 1)",
            "rsi(close, 2)",
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

#[test]
fn v3_core_fixture_lowers_names_constants_declaration_and_na_to_canonical_hir() {
    let source = include_str!("../../../../tests/fixtures/legacy/v3/runtime/core_legacy.pine");
    let analysis = analyze_production(source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.as_ref().expect("v3 core HIR");
    let value = hir
        .symbols
        .iter()
        .find(|symbol| symbol.name == "value")
        .expect("inferred v3 value symbol");
    assert_eq!(
        value.pine_type,
        PineType::new(Qualifier::Series, ValueKind::Float)
    );

    let translations = analysis
        .compatibility
        .legacy_translations
        .iter()
        .map(|translation| {
            (
                translation.source_feature.as_str(),
                translation.canonical_feature.as_str(),
            )
        })
        .collect::<Vec<_>>();
    for expected in [
        ("study", "indicator"),
        ("integer", "input.int"),
        ("input", "input.int"),
        ("ema", "ta.ema"),
        ("sma", "ta.sma"),
        ("red", "color.red"),
        ("color", "color.new"),
        ("histogram", "plot.style_histogram"),
        ("n", "bar_index"),
        ("interval", "timeframe.multiplier"),
        ("blue", "color.blue"),
        ("gray", "color.gray"),
        ("dotted", "hline.style_dotted"),
    ] {
        assert!(translations.contains(&expected), "missing {expected:?}");
    }
    assert_eq!(
        analysis
            .compatibility
            .legacy_emulations
            .iter()
            .filter(|emulation| emulation.feature == "v3.untyped_na")
            .count(),
        1
    );
}

#[test]
fn selected_pre_v4_constants_resolve_only_through_v3() {
    let aliases = [
        ("aqua", "color.aqua"),
        ("black", "color.black"),
        ("blue", "color.blue"),
        ("fuchsia", "color.fuchsia"),
        ("gray", "color.gray"),
        ("green", "color.green"),
        ("lime", "color.lime"),
        ("maroon", "color.maroon"),
        ("navy", "color.navy"),
        ("olive", "color.olive"),
        ("orange", "color.orange"),
        ("purple", "color.purple"),
        ("red", "color.red"),
        ("silver", "color.silver"),
        ("teal", "color.teal"),
        ("white", "color.white"),
        ("yellow", "color.yellow"),
        ("area", "plot.style_area"),
        ("areabr", "plot.style_areabr"),
        ("circles", "plot.style_circles"),
        ("columns", "plot.style_columns"),
        ("cross", "plot.style_cross"),
        ("histogram", "plot.style_histogram"),
        ("line", "plot.style_line"),
        ("linebr", "plot.style_linebr"),
        ("stepline", "plot.style_stepline"),
        ("dashed", "hline.style_dashed"),
        ("dotted", "hline.style_dotted"),
        ("solid", "hline.style_solid"),
        ("sunday", "dayofweek.sunday"),
        ("monday", "dayofweek.monday"),
        ("tuesday", "dayofweek.tuesday"),
        ("wednesday", "dayofweek.wednesday"),
        ("thursday", "dayofweek.thursday"),
        ("friday", "dayofweek.friday"),
        ("saturday", "dayofweek.saturday"),
        ("period", "timeframe.period"),
        ("isdaily", "timeframe.isdaily"),
        ("isdwm", "timeframe.isdwm"),
        ("isintraday", "timeframe.isintraday"),
        ("isminutes", "timeframe.isminutes"),
        ("isseconds", "timeframe.isseconds"),
        ("ismonthly", "timeframe.ismonthly"),
        ("isweekly", "timeframe.isweekly"),
        ("interval", "timeframe.multiplier"),
        ("ticker", "syminfo.ticker"),
        ("tickerid", "syminfo.tickerid"),
        ("n", "bar_index"),
        ("bool", "input.bool"),
        ("float", "input.float"),
        ("integer", "input.int"),
        ("resolution", "input.timeframe"),
        ("session", "input.session"),
        ("source", "input.source"),
        ("string", "input.string"),
        ("symbol", "input.symbol"),
    ];

    for version in 1..=3 {
        for (alias, canonical) in aliases {
            if alias == "isseconds" && version < 3 {
                continue;
            }
            let analysis = analyze_catalog_without_admission(&format!(
                "//@version={version}\nvalue = {alias}\n"
            ));
            assert!(
                analysis.diagnostics.is_empty(),
                "v{version} {alias}: {:?}",
                analysis.diagnostics
            );
            assert!(
                analysis
                    .compatibility
                    .legacy_translations
                    .iter()
                    .any(|translation| translation.source_feature == alias
                        && translation.canonical_feature == canonical)
            );
        }
    }

    for version in 4..=6 {
        for (alias, _) in aliases {
            let analysis = analyze_catalog_without_admission(&format!(
                "//@version={version}\nvalue = {alias}\n"
            ));
            assert!(!analysis.diagnostics.is_empty(), "v{version} {alias}");
            assert!(analysis.compatibility.legacy_translations.is_empty());
        }
    }
}

#[test]
fn v3_color_helper_is_versioned_and_user_symbols_still_shadow_other_aliases() {
    for version in 1..=3 {
        let analysis = analyze_catalog_without_admission(&format!(
            "//@version={version}\nshade = color(red, 50)\n"
        ));
        assert!(
            analysis.diagnostics.is_empty(),
            "v{version}: {:?}",
            analysis.diagnostics
        );
        assert!(
            analysis
                .compatibility
                .legacy_translations
                .iter()
                .any(|translation| translation.source_feature == "color"
                    && translation.canonical_feature == "color.new")
        );
    }

    let shadowed = analyze_production(include_str!(
        "../../../../tests/fixtures/legacy/v3/sema/shadowing.pine"
    ));
    assert!(
        shadowed.diagnostics.is_empty(),
        "{:?}",
        shadowed.diagnostics
    );
    for name in [
        "ema",
        "red",
        "n",
        "interval",
        "integer",
        "histogram",
        "dotted",
        "monday",
        "period",
        "isdaily",
        "ticker",
        "tickerid",
    ] {
        assert!(
            shadowed
                .compatibility
                .legacy_translations
                .iter()
                .all(|translation| translation.source_feature != name),
            "shadowed {name} was translated"
        );
    }
}

#[test]
fn v3_declaration_input_and_output_signatures_stay_version_specific() {
    let adapted = analyze_production(
        "//@version=3\nstudy(\"adapted output\")\nplot(close, color=red, style=5, transp=25)\n",
    );
    assert!(adapted.diagnostics.is_empty(), "{:?}", adapted.diagnostics);
    for feature in ["plot.transp", "plot.numeric_style"] {
        assert!(
            adapted
                .compatibility
                .legacy_emulations
                .iter()
                .any(|item| { item.feature == feature && item.behavior.contains("Pine v3") })
        );
    }

    for source in [
        "//@version=3\nstudy(\"too wide\", \"wide\", false, 2, true)\nplot(close)\n",
        "//@version=3\nstudy(\"format\", format=format.price)\nplot(close)\n",
        "//@version=3\nstudy(\"input\")\nx=input(1, tooltip=\"new\")\nplot(x)\n",
        "//@version=3\nstudy(\"plot\")\nplot(close, display=display.all)\n",
        include_str!(
            "../../../../tests/fixtures/legacy/v3/unsupported/later_signature_arguments.pine"
        ),
    ] {
        let analysis = analyze_production(source);
        assert!(!analysis.diagnostics.is_empty(), "{source}");
        assert!(analysis.hir.is_none());
    }

    let v4 = analyze_production(
        "//@version=4\nstudy(\"v4\", format=format.price)\nx=input(1, tooltip=\"ok\")\nplot(x, display=display.all)\n",
    );
    assert!(v4.diagnostics.is_empty(), "{:?}", v4.diagnostics);
}

#[test]
fn v3_untyped_na_infers_only_one_stable_scalar_type() {
    let scalar = analyze_production(
        r#"//@version=3
study("v3 scalar na")
i = na
i := 1
f = na
f := close
b = na
b := true
s = na
s := "text"
c = na
c := red
plot(i + f + (b ? 1 : 0))
"#,
    );
    assert!(scalar.diagnostics.is_empty(), "{:?}", scalar.diagnostics);
    let hir = scalar.hir.expect("scalar v3 HIR");
    for (name, expected) in [
        ("i", PineType::new(Qualifier::Const, ValueKind::Int)),
        ("f", PineType::new(Qualifier::Series, ValueKind::Float)),
        ("b", PineType::new(Qualifier::Const, ValueKind::Bool)),
        ("s", PineType::new(Qualifier::Const, ValueKind::String)),
        ("c", PineType::new(Qualifier::Const, ValueKind::Color)),
    ] {
        assert_eq!(
            hir.symbols
                .iter()
                .find(|symbol| symbol.name == name)
                .expect(name)
                .pine_type,
            expected,
            "{name}"
        );
    }

    for source in [
        include_str!("../../../../tests/fixtures/legacy/v3/unsupported/untyped_na_unresolved.pine"),
        include_str!("../../../../tests/fixtures/legacy/v3/unsupported/untyped_na_collection.pine"),
        include_str!("../../../../tests/fixtures/legacy/v3/unsupported/untyped_na_conflict.pine"),
    ] {
        let analysis = analyze_production(source);
        assert_eq!(
            diagnostic_codes(&analysis),
            vec!["E_LEGACY_V3_NA_INFERENCE"],
            "{source}: {:?}",
            analysis.diagnostics
        );
        assert!(analysis.hir.is_none());
    }

    for source in [
        "//@version=4\nstudy(\"strict v4\")\nvalue=na\nvalue:=close\nplot(close)\n",
        "//@version=6\nindicator(\"strict v6\")\nvalue=na\nvalue:=close\nplot(close)\n",
    ] {
        let analysis = analyze_production(source);
        assert!(!analysis.diagnostics.is_empty());
        assert!(
            diagnostic_codes(&analysis)
                .iter()
                .all(|code| *code != "E_LEGACY_V3_NA_INFERENCE")
        );
        assert!(analysis.compatibility.legacy_emulations.is_empty());
    }
}

#[test]
fn v2_declaration_graph_preserves_symbol_identity_and_stable_current_order() {
    let analysis = analyze_production(include_str!(
        "../../../../tests/fixtures/legacy/v2/runtime/core_legacy.pine"
    ));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.as_ref().expect("v2 HIR");
    let symbol_names = hir
        .symbols
        .iter()
        .map(|symbol| (symbol.id, symbol.name.as_str()))
        .collect::<std::collections::HashMap<_, _>>();
    let declaration_names = hir
        .statements
        .iter()
        .filter_map(|statement| match statement.kind {
            HirStmtKind::Decl { symbol, .. } => symbol_names.get(&symbol).copied(),
            _ => None,
        })
        .collect::<Vec<_>>();
    let later = declaration_names
        .iter()
        .position(|name| *name == "laterCurrent")
        .expect("later declaration");
    let consumer = declaration_names
        .iter()
        .position(|name| *name == "currentForward")
        .expect("forward consumer");
    assert!(later < consumer, "{declaration_names:?}");

    let self_symbol = hir
        .symbols
        .iter()
        .find(|symbol| symbol.name == "selfSeries")
        .expect("self symbol");
    assert!(self_symbol.series_id.is_some());
    let self_decl = hir
        .statements
        .iter()
        .find_map(|statement| match &statement.kind {
            HirStmtKind::Decl { symbol, value } if *symbol == self_symbol.id => Some(value),
            _ => None,
        })
        .expect("self declaration");
    assert!(hir_expr_contains_history_symbol(self_decl, self_symbol.id));

    for feature in [
        "v2.self_reference",
        "v2.forward_reference",
        "v2.bool_arithmetic",
        "v2.numeric_to_bool",
    ] {
        assert!(
            analysis
                .compatibility
                .legacy_emulations
                .iter()
                .any(|emulation| emulation.feature == feature),
            "missing {feature}"
        );
    }
    assert!(hir_contains_call(hir, "float"));
    assert!(hir_contains_call(hir, "bool"));
}

#[test]
fn v2_declaration_graph_rejects_cycles_barriers_unsafe_calls_and_limits_once() {
    for (source, expected) in [
        (
            include_str!("../../../../tests/fixtures/legacy/v2/unsupported/reference_cycle.pine"),
            "E_LEGACY_REFERENCE_CYCLE",
        ),
        (
            include_str!(
                "../../../../tests/fixtures/legacy/v2/unsupported/forward_reference_barrier.pine"
            ),
            "E_LEGACY_FORWARD_REFERENCE_UNSAFE",
        ),
        (
            include_str!("../../../../tests/fixtures/legacy/v2/unsupported/unsafe_graph_call.pine"),
            "E_LEGACY_REFERENCE_GRAPH_UNSAFE",
        ),
    ] {
        let analysis = analyze_production(source);
        assert_eq!(diagnostic_codes(&analysis), vec![expected]);
        assert!(analysis.hir.is_none());
    }

    let mut oversized = String::from(
        "//@version=2\nstudy(\"oversized graph\")\nfirst = node256 + 1\nnode0 = close\n",
    );
    for index in 1..=256 {
        oversized.push_str(&format!("node{index} = node{} + 1\n", index - 1));
    }
    oversized.push_str("plot(first)\n");
    let analysis = analyze_production(&oversized);
    assert_eq!(
        diagnostic_codes(&analysis),
        vec!["E_LEGACY_REFERENCE_GRAPH_LIMIT"]
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn v2_declaration_graph_enforces_the_edge_limit_independently() {
    let mut oversized = String::from("//@version=2\nstudy(\"oversized edge graph\")\n");
    oversized.push_str("first = ");
    for index in 1..=100 {
        if index > 1 {
            oversized.push_str(" + ");
        }
        oversized.push_str(&format!("node{index}"));
    }
    oversized.push('\n');
    oversized.push_str("node0 = close\n");
    for index in 1..=100 {
        oversized.push_str(&format!("node{index} = node0"));
        for dependency in 1..index {
            oversized.push_str(&format!(" + node{dependency}"));
        }
        oversized.push('\n');
    }
    oversized.push_str("plot(first)\n");

    let analysis = analyze_production(&oversized);
    assert_eq!(
        diagnostic_codes(&analysis),
        vec!["E_LEGACY_REFERENCE_GRAPH_LIMIT"]
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn v2_bool_arithmetic_and_pre_v6_numeric_conditions_keep_version_boundaries() {
    let v2 = analyze_production(
        "//@version=2\nstudy(\"coercions\")\nb=close>open\nplot(b + true)\nplot((close-open) ? 1 : 0)\n",
    );
    assert!(v2.diagnostics.is_empty(), "{:?}", v2.diagnostics);
    assert!(hir_contains_call(v2.hir.as_ref().expect("v2 HIR"), "float"));
    assert!(hir_contains_call(v2.hir.as_ref().expect("v2 HIR"), "bool"));

    let v3_bool = analyze_production(include_str!(
        "../../../../tests/fixtures/legacy/v3/unsupported/bool_arithmetic.pine"
    ));
    assert_eq!(diagnostic_codes(&v3_bool), vec!["E_OPERATOR_TYPE"]);
    let v3_numeric =
        analyze_production("//@version=3\nstudy(\"numeric condition\")\nplot(close ? 1 : 0)\n");
    assert!(
        v3_numeric.diagnostics.is_empty(),
        "{:?}",
        v3_numeric.diagnostics
    );
    assert!(hir_contains_call(
        v3_numeric.hir.as_ref().expect("v3 numeric condition HIR"),
        "bool"
    ));

    let v6 = analyze_production(include_str!(
        "../../../../tests/fixtures/legacy/v6/unsupported/numeric_condition.pine"
    ));
    assert_eq!(diagnostic_codes(&v6), vec!["E_CONDITION_TYPE"]);
    assert!(v6.hir.is_none());
}

#[test]
fn implicit_v1_matches_explicit_v2_only_for_the_claimed_shared_profile() {
    let v1 = analyze_production(include_str!(
        "../../../../tests/fixtures/legacy/v1/runtime/shared_v1.pine"
    ));
    let v2 = analyze_production(include_str!(
        "../../../../tests/fixtures/legacy/v2/runtime/shared_v2.pine"
    ));
    assert!(v1.diagnostics.is_empty(), "{:?}", v1.diagnostics);
    assert!(v2.diagnostics.is_empty(), "{:?}", v2.diagnostics);
    assert_eq!(
        v1.compatibility.language_version_origin,
        crate::VersionOrigin::ImplicitV1
    );
    let mut v1_hir = v1.hir.expect("v1 HIR");
    let mut v2_hir = v2.hir.expect("v2 HIR");
    v1_hir.language_version = None;
    v2_hir.language_version = None;
    assert_eq!(v1_hir, v2_hir);
}

fn hir_expr_contains_history_symbol(expr: &HirExpr, symbol: pine_ir::SymbolId) -> bool {
    match &expr.kind {
        HirExprKind::History { expr, .. } => {
            matches!(expr.kind, HirExprKind::Symbol(candidate) if candidate == symbol)
                || hir_expr_contains_history_symbol(expr, symbol)
        }
        HirExprKind::Unary { expr, .. } => hir_expr_contains_history_symbol(expr, symbol),
        HirExprKind::Binary { left, right, .. } => {
            hir_expr_contains_history_symbol(left, symbol)
                || hir_expr_contains_history_symbol(right, symbol)
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            hir_expr_contains_history_symbol(condition, symbol)
                || hir_expr_contains_history_symbol(then_expr, symbol)
                || hir_expr_contains_history_symbol(else_expr, symbol)
        }
        HirExprKind::Call { args, .. } => args
            .iter()
            .any(|arg| hir_expr_contains_history_symbol(&arg.value, symbol)),
        _ => false,
    }
}

fn hir_contains_call(hir: &pine_ir::HirProgram, expected: &str) -> bool {
    hir.statements
        .iter()
        .any(|statement| match &statement.kind {
            HirStmtKind::Expr(expr)
            | HirStmtKind::Decl { value: expr, .. }
            | HirStmtKind::Reassign { value: expr, .. } => hir_expr_contains_call(expr, expected),
            _ => false,
        })
}

fn hir_expr_contains_call(expr: &HirExpr, expected: &str) -> bool {
    match &expr.kind {
        HirExprKind::Call { callee, args, .. } => {
            callee == expected
                || args
                    .iter()
                    .any(|arg| hir_expr_contains_call(&arg.value, expected))
        }
        HirExprKind::Unary { expr, .. } | HirExprKind::History { expr, .. } => {
            hir_expr_contains_call(expr, expected)
        }
        HirExprKind::Binary { left, right, .. } => {
            hir_expr_contains_call(left, expected) || hir_expr_contains_call(right, expected)
        }
        HirExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            hir_expr_contains_call(condition, expected)
                || hir_expr_contains_call(then_expr, expected)
                || hir_expr_contains_call(else_expr, expected)
        }
        _ => false,
    }
}
