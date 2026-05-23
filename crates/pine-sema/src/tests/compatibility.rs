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
    let analysis =
        analyze("plot(request.security(\"NYSE:IBM\", timeframe.period, ta.sma(close, 2)))\n");

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
        analyze("x = request.security(\"NYSE:IBM\", timeframe.period, math.max(close, open))\n");

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
