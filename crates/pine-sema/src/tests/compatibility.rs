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
