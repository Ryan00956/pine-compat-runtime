use super::*;

#[test]
fn reports_supported_phase_1_calls() {
    let analysis = analyze("plot(ta.sma(close, 20))\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "plot")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.sma")
    );
}

#[test]
fn accepts_same_context_request_security() {
    let analysis = analyze("plot(request.security(syminfo.tickerid, timeframe.period, close))\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_global_scalar_varip_declaration() {
    let analysis = analyze("varip x = 0\nx := x + 1\nplot(x)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "varip")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("varip script should lower");
    let symbol = hir
        .symbols
        .iter()
        .find(|symbol| symbol.name == "x")
        .expect("x symbol should exist");
    assert_eq!(symbol.persistence, PersistenceKind::Varip);
    assert_eq!(symbol.var_slot_id, Some(VarSlotId(0)));
}

#[test]
fn accepts_local_scalar_varip_declaration() {
    let analysis = analyze("if close > open\n    varip x = 0\n    x := x + 1\n    plot(x)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "varip")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("local varip script should lower");
    let symbol = hir
        .symbols
        .iter()
        .find(|symbol| symbol.name == "x")
        .expect("x symbol should exist");
    assert_eq!(symbol.persistence, PersistenceKind::Varip);
    assert_eq!(symbol.var_slot_id, Some(VarSlotId(0)));
}

#[test]
fn accepts_scalar_array_varip_declarations() {
    let analysis = analyze(
        r#"varip floats = array.new_float(0)
varip ints = array.new_int(0)
varip flags = array.new_bool(0)
varip words = array.new_string(0)
varip colors = array.new_color(0)
plot(array.size(floats) + array.size(ints) + array.size(flags) + array.size(words) + array.size(colors))
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("array varip script should lower");
    let symbols: Vec<_> = hir
        .symbols
        .iter()
        .filter(|symbol| {
            matches!(
                symbol.name.as_str(),
                "floats" | "ints" | "flags" | "words" | "colors"
            )
        })
        .collect();
    assert_eq!(symbols.len(), 5);
    assert!(
        symbols
            .iter()
            .all(|symbol| symbol.persistence == PersistenceKind::Varip)
    );
    assert!(symbols.iter().all(|symbol| symbol.var_slot_id.is_some()));
}

#[test]
fn rejects_tuple_varip_declaration() {
    let analysis = analyze("varip values = [1, 2]\nplot(close)\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(analysis.compatibility.unsupported[0].feature, "varip");
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("tuples")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_drawing_id_varip_declarations() {
    for source in [
        "varip id = label.new(bar_index, high, \"x\")\nplot(close)\n",
        "varip id = line.new(bar_index, low, bar_index, high)\nplot(close)\n",
        "varip id = box.new(bar_index, high, bar_index, low)\nplot(close)\n",
        "varip id = table.new(position.top_right, 1, 1)\nplot(close)\n",
    ] {
        let analysis = analyze(source);

        assert_eq!(analysis.compatibility.unsupported.len(), 1, "{source}");
        assert_eq!(analysis.compatibility.unsupported[0].feature, "varip");
        assert!(
            analysis.compatibility.unsupported[0]
                .reason
                .contains("drawing object ids"),
            "{:?}",
            analysis.compatibility.unsupported
        );
        assert!(analysis.hir.is_none());
    }
}

#[test]
fn rejects_reassigning_drawing_id_into_na_varip() {
    let analysis = analyze("varip id = na\nid := label.new(bar_index, high, \"x\")\nplot(close)\n");

    assert!(analysis.hir.is_none());
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_ASSIGN_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
}

#[test]
fn accepts_provider_backed_same_timeframe_request_security_source() {
    let analysis = analyze("plot(request.security(\"NYSE:IBM\", timeframe.period, close))\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_provider_backed_same_timeframe_request_security_ta_variable() {
    let analysis = analyze(
        "plot(request.security(\"NYSE:IBM\", timeframe.period, ta.accdist + ta.iii + ta.obv + ta.pvt + ta.wvad))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_provider_backed_same_timeframe_request_security_expression() {
    let analysis =
        analyze("plot(request.security(\"NYSE:IBM\", timeframe.period, close + open))\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_provider_backed_same_timeframe_request_security_ta() {
    let analysis = analyze(
        "plot(request.security(\"NYSE:IBM\", timeframe.period, ta.sma(close, 2) + ta.ema(close, 2) + ta.dema(close, 2) + ta.tema(close, 2) + ta.rma(close, 2) + ta.cum(close) + ta.rsi(close, 3) + ta.tsi(close, 2, 3) + ta.cmo(close, 3) + ta.cci(close, 3) + ta.cog(close, 3) + nz(ta.bop()) + nz(ta.ao()) + ta.max(close) + ta.min(open) + ta.mfi(close, 3) + ta.stoch(close, high, low, 3) + ta.wpr(3) + ta.sar(0.02, 0.02, 0.2) + ta.tr() + nz(ta.tr(false)) + ta.atr(3) + ta.highest(high, 3) + nz(ta.highestbars(close, 3)) - ta.lowest(low, 3) + nz(ta.lowestbars(close, 3)) + ta.change(close) + ta.mom(close, 2) + ta.roc(close, 2) + ta.range(close, 3) + ta.dev(close, 3) + ta.vwap(close) + ta.bbw(close, 3, 2) + ta.kcw(close, 3, 2, false) + nz(ta.pivothigh(high, 1, 1)) - nz(ta.pivotlow(low, 1, 1)) + ta.correlation(close, high, 3) + ta.covariance(close, high, 3) + ta.median(close, 3) + ta.mode(close, 3) + ta.percentile_nearest_rank(close, 3, 50) + ta.percentile_linear_interpolation(close, 3, 50) + ta.percentrank(close, 3) + ta.stdev(close, 3) + ta.stdev(close, 3, false) + ta.variance(close, 3) + ta.variance(close, 3, false) + ta.wma(close, 3) + ta.vwma(close, 3) + ta.swma(close) + ta.hma(close, 4) + ta.alma(close, 4, 0.85, 6) + ta.linreg(close, 3, 0) + (ta.rising(close, 2) ? 1 : 0) - (ta.falling(close, 2) ? 1 : 0) + nz(ta.barssince(close > open)) + nz(ta.valuewhen(close > open, close, 1)) + (ta.cross(close, 20.5) ? 1 : 0) + (ta.crossover(close, 20.5) ? 1 : 0) - (ta.crossunder(close - time / 60000.0, 19.5) ? 1 : 0)))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_provider_backed_request_security_math_extremes() {
    let analysis = analyze(
        "plot(request.security(\"NYSE:IBM\", timeframe.period, math.max(close, open) - math.min(close, open)))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_provider_backed_request_security_supported_math_calls() {
    let analysis = analyze(
        "plot(request.security(\"NYSE:IBM\", timeframe.period, math.abs(open - close) + math.avg(open, close) + math.floor(close) + math.ceil(open) + math.trunc(high) + math.sqrt(close) + math.cbrt(close) + math.log(close) + math.log10(close) + math.exp(1) + math.acos(0.5) + math.asin(0.5) + math.atan(close) + math.sign(close - open) + math.todegrees(close) + math.toradians(open) + math.sin(close) + math.cos(open) + math.tan(0) + math.pow(close, 2) + math.hypot(close, open) + math.round(close / 3, 2) + math.round_to_mintick(close + 0.006) + nz(math.sum(close, 2))))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_same_context_request_security_math_extremes() {
    let analysis = analyze(
        "plot(request.security(syminfo.tickerid, timeframe.period, math.max(close, open) - math.min(close, open)))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn accepts_provider_backed_higher_timeframe_request_security() {
    let analysis = analyze("plot(request.security(\"NYSE:IBM\", \"5\", ta.sma(close, 2)))\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "request.security")
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn rejects_invalid_timeframe_request_security() {
    let analysis = analyze("x = request.security(\"AAPL\", close, close)\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_UNSUPPORTED_FEATURE")
    );
}

#[test]
fn rejects_provider_request_security_unsupported_call() {
    let analysis =
        analyze("x = request.security(\"NYSE:IBM\", timeframe.period, math.random(0, 1, 7))\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
}

#[test]
fn rejects_provider_request_security_tuple_ta_call() {
    let analysis = analyze(
        "x = request.security(\"NYSE:IBM\", timeframe.period, ta.vwap(close, false, 2.0))\n",
    );

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
}

#[test]
fn rejects_provider_request_security_local_variable_expression() {
    let analysis =
        analyze("src = close\nx = request.security(\"NYSE:IBM\", timeframe.period, src)\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
}

#[test]
fn rejects_higher_timeframe_request_security_local_variable_expression() {
    let analysis = analyze("src = close\nx = request.security(\"NYSE:IBM\", \"5\", src)\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
}

#[test]
fn rejects_request_security_side_effect_expression() {
    let analysis =
        analyze("x = request.security(syminfo.tickerid, timeframe.period, plot(close))\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("side-effecting requested expressions")
    );
}

#[test]
fn rejects_request_security_alertcondition_side_effect_expression() {
    let analysis = analyze(
        "x = request.security(syminfo.tickerid, timeframe.period, alertcondition(true, \"A\", \"B\"))\n",
    );

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("side-effecting requested expressions")
    );
}

#[test]
fn rejects_request_security_alert_side_effect_expression() {
    let analysis =
        analyze("x = request.security(syminfo.tickerid, timeframe.period, alert(\"A\"))\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("side-effecting requested expressions")
    );
}

#[test]
fn rejects_other_request_variants() {
    let analysis = analyze("x = request.financial(syminfo.tickerid, \"TOTAL_REVENUE\", \"FQ\")\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.financial"
    );
}

#[test]
fn rejects_request_security_lower_tf_api() {
    let analysis = analyze("x = request.security_lower_tf(\"NYSE:IBM\", \"30S\", close)\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security_lower_tf"
    );
}

#[test]
fn compile_cache_reuses_analysis_for_identical_source() {
    let source = SourceFile::new("test.pine", "plot(close)\n");
    let mut cache = CompileCache::new();

    let first = cache.analyze(&source);
    let second = cache.analyze(&source);

    assert_eq!(first, second);
    assert_eq!(
        cache.stats(),
        CompileCacheStats {
            entries: 1,
            hits: 1,
            misses: 1,
        }
    );
}

#[test]
fn compile_cache_keys_by_source_name_and_text() {
    let mut cache = CompileCache::new();

    cache.analyze(&SourceFile::new("one.pine", "plot(close)\n"));
    cache.analyze(&SourceFile::new("two.pine", "plot(close)\n"));
    cache.analyze(&SourceFile::new("one.pine", "plot(open)\n"));

    assert_eq!(
        cache.stats(),
        CompileCacheStats {
            entries: 3,
            hits: 0,
            misses: 3,
        }
    );
}

#[test]
fn compile_cache_keys_by_source_graph_library_sources() {
    let root = SourceFile::new("root.pine", "plot(close)\n");
    let library_one = SourceFile::new("lib.pine", "library(\"one\")\n");
    let library_two = SourceFile::new("lib.pine", "library(\"two\")\n");
    let first_input = AnalysisInput::with_library_sources(
        root.clone(),
        vec![("user/lib/1".to_owned(), library_one)],
    )
    .expect("first input");
    let second_input =
        AnalysisInput::with_library_sources(root, vec![("user/lib/1".to_owned(), library_two)])
            .expect("second input");
    let mut cache = CompileCache::new();

    cache.analyze_input(&first_input);
    cache.analyze_input(&first_input);
    cache.analyze_input(&second_input);

    assert_eq!(
        cache.stats(),
        CompileCacheStats {
            entries: 2,
            hits: 1,
            misses: 2,
        }
    );
}

#[test]
fn source_graph_input_reports_duplicate_library_key() {
    let error = AnalysisInput::with_library_sources(
        SourceFile::new("root.pine", "plot(close)\n"),
        vec![
            (
                "user/lib/1".to_owned(),
                SourceFile::new("one.pine", "library(\"one\")\n"),
            ),
            (
                "user/lib/1".to_owned(),
                SourceFile::new("two.pine", "library(\"two\")\n"),
            ),
        ],
    )
    .expect_err("duplicate keys should fail");

    assert_eq!(
        error,
        SourceGraphError::DuplicateLibraryKey {
            key: "user/lib/1".to_owned()
        }
    );
}

fn analyze_with_libraries(root: &str, libraries: Vec<(&str, &str)>) -> Analysis {
    let input = AnalysisInput::with_library_sources(
        SourceFile::new("root.pine", root),
        libraries
            .into_iter()
            .map(|(key, source)| {
                (
                    key.to_owned(),
                    SourceFile::new(format!("{key}.pine"), source),
                )
            })
            .collect(),
    )
    .expect("analysis input");
    crate::analyze_input(&input)
}

fn diagnostic_codes(analysis: &Analysis) -> Vec<&str> {
    analysis
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

#[test]
fn import_reports_missing_library_source() {
    let analysis = analyze_with_libraries("import user/lib/1 as lib\nplot(close)\n", vec![]);

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_MISSING_LIBRARY"), "{codes:?}");
}

#[test]
fn import_reports_duplicate_alias() {
    let analysis = analyze_with_libraries(
        "import user/one/1 as lib\nimport user/two/1 as lib\nplot(close)\n",
        vec![
            ("user/one/1", "library(\"one\")\n"),
            ("user/two/1", "library(\"two\")\n"),
        ],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_DUPLICATE_ALIAS"), "{codes:?}");
}

#[test]
fn import_reports_invalid_library_declaration() {
    let analysis = analyze_with_libraries(
        "import user/lib/1 as lib\nplot(close)\n",
        vec![("user/lib/1", "export value = 1\n")],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_INVALID_LIBRARY"), "{codes:?}");
}

#[test]
fn import_reports_duplicate_exports() {
    let analysis = analyze_with_libraries(
        "import user/lib/1 as lib\nplot(close)\n",
        vec![(
            "user/lib/1",
            "library(\"lib\")\nexport value = 1\nexport value = 2\n",
        )],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_DUPLICATE_EXPORT"), "{codes:?}");
}

#[test]
fn import_reports_dependency_cycle() {
    let analysis = analyze_with_libraries(
        "import user/one/1 as one\nplot(close)\n",
        vec![
            ("user/one/1", "library(\"one\")\nimport user/two/1 as two\n"),
            ("user/two/1", "library(\"two\")\nimport user/one/1 as one\n"),
        ],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_CYCLE"), "{codes:?}");
}

#[test]
fn import_reports_unknown_export_and_private_access() {
    let analysis = analyze_with_libraries(
        "import user/lib/1 as lib\nplot(lib.missing)\nplot(lib.private)\n",
        vec![(
            "user/lib/1",
            "library(\"lib\")\nexport value = 1\nprivate = 2\n",
        )],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_UNKNOWN_EXPORT"), "{codes:?}");
    assert!(codes.contains(&"E_IMPORT_PRIVATE_SYMBOL"), "{codes:?}");
}

#[test]
fn import_accepts_exported_constant_and_pure_function_subset() {
    let analysis = analyze_with_libraries(
        "import user/lib/1 as lib\nplot(lib.scale(close) + lib.offset)\n",
        vec![(
            "user/lib/1",
            "library(\"lib\")\nexport offset = 2\nexport scale(value) => value * offset\n",
        )],
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn import_reports_missing_alias_for_executable_subset() {
    let analysis = analyze_with_libraries(
        "import user/lib/1\nplot(close)\n",
        vec![("user/lib/1", "library(\"lib\")\nexport value = 1\n")],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_ALIAS_REQUIRED"), "{codes:?}");
}

#[test]
fn import_rejects_exported_series_constant() {
    let analysis = analyze_with_libraries(
        "import user/lib/1 as lib\nplot(close)\n",
        vec![("user/lib/1", "library(\"lib\")\nexport value = close\n")],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_CONST_VALUE"), "{codes:?}");
}

#[test]
fn import_rejects_exported_function_side_effects() {
    let analysis = analyze_with_libraries(
        "import user/lib/1 as lib\nplot(close)\n",
        vec![(
            "user/lib/1",
            "library(\"lib\")\nexport draw(value) => plot(value)\n",
        )],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(
        codes.contains(&"E_IMPORT_FUNCTION_SIDE_EFFECT"),
        "{codes:?}"
    );
}

#[test]
fn import_rejects_recursive_exported_functions() {
    let analysis = analyze_with_libraries(
        "import user/lib/1 as lib\nplot(lib.loop(close))\n",
        vec![(
            "user/lib/1",
            "library(\"lib\")\nexport loop(value) => value > 0 ? loop(value - 1) : value\n",
        )],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_RECURSIVE_FUNCTION"), "{codes:?}");
}

#[test]
fn import_rejects_imported_user_type_constructors() {
    let analysis = analyze_with_libraries(
        "import user/udt/1 as lib\np = lib.Point.new(close)\nplot(p.x)\n",
        vec![("user/udt/1", "library(\"udt\")\ntype Point\n    float x\n")],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_IMPORT_UNKNOWN_EXPORT"), "{codes:?}");
    assert!(analysis.hir.is_none());
}

#[test]
fn import_rejects_imported_user_methods() {
    let analysis = analyze_with_libraries(
        "import user/methods/1 as lib\ntype Point\n    float x\np = Point.new(close)\nplot(p.shift(1))\n",
        vec![(
            "user/methods/1",
            "library(\"methods\")\ntype Point\n    float x\nmethod shift(Point p, float delta) => p.x + delta\n",
        )],
    );

    let codes = diagnostic_codes(&analysis);
    assert!(codes.contains(&"E_UNKNOWN_METHOD"), "{codes:?}");
    assert!(analysis.hir.is_none());
}

#[test]
fn compile_cache_clear_drops_entries_and_stats() {
    let source = SourceFile::new("test.pine", "plot(close)\n");
    let mut cache = CompileCache::new();

    cache.analyze(&source);
    cache.analyze(&source);
    cache.clear();

    assert_eq!(
        cache.stats(),
        CompileCacheStats {
            entries: 0,
            hits: 0,
            misses: 0,
        }
    );
}
